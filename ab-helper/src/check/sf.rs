use std::{
	thread,
	time::Duration,
};

use kuchikiki::NodeRef;

use super::prelude::*;

/// Search SourceForge sources
#[derive(Clap)]
pub struct Sf {
	/// Name of the project
	name: String,
	/// The File name prefix to select download names with; defaults to project name
	prefix: Option<String>,
	/// Filter file names with a glob pattern
	#[arg(short, long, group = "file-filter")]
	glob: Option<String>,
	/// Filter file names with a regular expression
	#[arg(short, long, group = "file-filter")]
	regex: Option<String>,
	/// The (top-level) directory to start the search from; the value is appended to sourceforge.net/projects/<name>/files/
	#[arg(long)]
	dir: Option<String>,

	/// Only traverse (top-level) directories matching the given glob pattern
	#[arg(short, long, group = "dir-filter")]
	dir_glob: Option<String>,
	/// Only traverse (top-level) directories matching the given regular expression
	#[arg(short = 'D', long, group = "dir-filter")]
	dir_regex: Option<String>,
}

enum Link {
	File(Version, String),
	Folder(Option<Version>, String),
}

fn find_latest(
	c: &Agent,
	url: Url,
	doc: NodeRef,
	prefix: &str,
	filter: &Filter,
	dir_filter: &Filter,
	allow_pre: bool,
) -> Result<Option<(Version, Url)>> {
	let mut latest = Version::MIN;
	let mut latest_url = Url::parse("https://example.com")?;

	let mut dirs = Vec::with_capacity(32);

	let links = doc.query("tr.folder, tr.file").filter_map(|row| {
		let s = row.query("span.name").next()?.inner_text();
		let is_file = row.is_class("file");
		if (is_file && !filter.is_match(&s)) || (!is_file && !dir_filter.is_match(&s)) {
			return None;
		}

		let a = row.query("th > a[href]").next()?;
		let href = a.href()?;

		if is_file {
			let (ver, _) = extract_version(&s, prefix)?;
			if !allow_pre && ver.is_pre() {
				trace!("skipping prerelease: {ver}");
				return None;
			}
			Some(Link::File(ver, href))
		} else {
			let dir_ver = s
				.strip_prefix(prefix)
				.map(|s| s.strip_prefix('-').unwrap_or(s))
				.unwrap_or(&s);
			let dir_ver = Version::parse(dir_ver);

			Some(Link::Folder(dir_ver, href))
		}
	});

	for link in links {
		match link {
			Link::File(ver, href) if ver > latest => {
				latest = ver;
				latest_url = url.join(&href)?;
			}
			Link::File(ver, ..) => trace!("ignoring lesser version {ver}; already have {latest}"),
			Link::Folder(dir_ver, href) => {
				dirs.push((dir_ver, href));
			}
		}
	}

	// Put largest versions first.
	dirs.sort_unstable_by(|a, b| b.0.cmp(&a.0));

	let mut ignore_versioned_dirs = false;
	for (dir_version, dir) in dirs {
		if ignore_versioned_dirs && dir_version.is_some() {
			debug!("skipping directory {dir} (have better version {latest})");
			continue;
		}

		// Preemptively sleep a tiny bit to not cause alarm.
		thread::sleep(Duration::from_millis(25));
		let url = url.join(&dir)?;
		let h = get(c, url.as_str())?;

		// Note: Not reusing dir_filter because it applies to top-level directories only.
		if let Some((v, u)) = find_latest(c, url, h, prefix, filter, &Filter::Any, allow_pre)? {
			ignore_versioned_dirs = ignore_versioned_dirs || dir_version.is_some();
			if v > latest {
				debug!("found version {v} at {u}");
				latest = v;
				latest_url = u;
			}
		}
	}

	if latest == Version::MIN {
		Ok(None)
	} else {
		Ok(Some((latest, latest_url)))
	}
}

pub fn run(x: Sf, c: Agent, allow_pre: bool) -> VersionResult {
	let (doc, url) = match &x.dir {
		None => {
			if x.dir_glob.is_none() && x.dir_regex.is_none() {
				// Try files/<name> since this is the general case
				let url = Url::parse(&format!(
					"https://sourceforge.net/projects/{}/files/{}/",
					x.name, x.name
				))?;

				if let Ok(doc) = get(&c, url.as_str()) {
					(doc, url)
				} else {
					let url = Url::parse(&format!(
						"https://sourceforge.net/projects/{}/files/",
						x.name
					))?;
					(get(&c, url.as_str())?, url)
				}
			} else {
				let url = Url::parse(&format!(
					"https://sourceforge.net/projects/{}/files/",
					x.name
				))?;
				(get(&c, url.as_str())?, url)
			}
		}
		Some(dir) => {
			let url = Url::parse(&format!(
				"https://sourceforge.net/projects/{}/files/{}/",
				x.name,
				dir.trim_end_matches('/').trim_start_matches('/')
			))?;
			(get(&c, url.as_str())?, url)
		}
	};

	let prefix = x.prefix.as_deref().unwrap_or(&x.name);
	let filter = x
		.glob
		.map(Filter::glob)
		.or_else(|| x.regex.map(Filter::regex))
		.transpose()?
		.unwrap_or(Filter::Any);

	let dir_filter = x
		.dir_regex
		.map(Filter::regex)
		.or_else(|| x.dir_glob.map(Filter::glob))
		.transpose()?
		.unwrap_or(Filter::Any);

	let (version, url) =
		find_latest(&c, url, doc, prefix, &filter, &dir_filter, allow_pre)?.ok_or(NotFound)?;
	version_result(version, url)
}

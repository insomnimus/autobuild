use super::prelude::*;

/// Search GNU releases
#[derive(Clap)]
pub struct Gnu {
	/// Name of the project
	name: String,
	/// File name prefix for a download
	prefix: Option<String>,
}

pub fn run(x: Gnu, c: Agent) -> VersionResult {
	let mut url_str = if x.name.starts_with("https://") || x.name.starts_with("http://") {
		x.name
	} else {
		format!("https://ftp.gnu.org/pub/gnu/{}", x.name)
	};
	if !url_str.ends_with('/') {
		url_str.push('/');
	}

	let url = Url::parse(&url_str)?;
	let prefix = x.prefix.as_deref().unwrap_or_else(|| {
		url_str
			.trim_end_matches('/')
			.rsplit('/')
			.next()
			.unwrap_or_default()
	});

	let h = get(&c, &url_str)?;

	let (_, version, href) = h
		.query("tr")
		.flat_map(|row| {
			let cells = row.query("td").collect::<Vec<_>>();
			if cells.len() < 4 {
				return None;
			}

			let a = cells[1].query("a").next()?;
			let href = a.href()?;
			let text = a.inner_text();
			let (_, s) = extract_version(&text, prefix)?;

			let date = cells[2].inner_text();
			if date.is_empty() {
				None
			} else {
				Some(((date, Ext::parse(&text)), s.to_owned(), href))
			}
		})
		.max_by(|a, b| a.0.cmp(&b.0))
		.ok_or(NotFound)?;

	version_result(version, url.join(&href)?)
}

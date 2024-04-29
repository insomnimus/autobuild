use super::prelude::*;

#[derive(Clap)]
/// Search OpenBSD sources
pub struct OpenBsd {
	/// Name of the project
	name: String,
	/// File name prefix to filter downloads, defaults to name
	prefix: Option<String>,
}

pub fn run(x: OpenBsd, c: Agent, allow_pre: bool) -> VersionResult {
	let prefix = x.prefix.as_deref().unwrap_or(&x.name);
	let url = format!("https://ftp.openbsd.org/pub/OpenBSD/{}/", x.name);
	let h = get(&c, &url)?;
	let (_, version, href) = h
		.query("a")
		.filter_map(|a| {
			let s = a.inner_text();
			let (v, s) = extract_version(&s, prefix)?;
			if !allow_pre && v.is_pre() {
				None
			} else {
				let href = a.href()?;
				Some((v, s.to_owned(), href))
			}
		})
		.max_by(|a, b| a.0.cmp(&b.0))
		.ok_or(NotFound)?;

	version_result(version, Url::parse(&url)?.join(&href)?)
}

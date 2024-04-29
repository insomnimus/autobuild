use super::prelude::*;

#[derive(Clap)]
/// Search Gnome sources
pub struct Gnome {
	/// Name of the project
	name: String,
	/// File name prefix to filter downloads, defaults to name
	prefix: Option<String>,
}

pub fn run(x: Gnome, c: Agent) -> VersionResult {
	let url = format!("https://download.gnome.org/sources/{}/", x.name);
	let prefix = x.prefix.as_deref().unwrap_or(&x.name);

	let h = get(&c, &url)?;
	let sel = "td.link > a[href][title]";
	let (_, dir) = h
		.query(sel)
		.filter_map(|a| {
			let href = a.href()?;
			let v = Version::parse(&a.attr_filter("title")?)?;
			Some((v, href))
		})
		.max_by(|a, b| a.0.cmp(&b.0))
		.ok_or(NotFound)?;

	let url = Url::parse(&url)?.join(&dir)?;
	let h = get(&c, url.as_str())?;

	let (_, version, href) = h
		.query(sel)
		.filter_map(|a| {
			let href = a.href()?;
			let s = a.attr_filter("title")?;
			let (v, s) = extract_version(&s, prefix)?;

			Some(((v, Ext::parse(s)), s.to_owned(), href))
		})
		.max_by(|a, b| a.0.cmp(&b.0))
		.ok_or(NotFound)?;

	version_result(version, url.join(&href)?)
}

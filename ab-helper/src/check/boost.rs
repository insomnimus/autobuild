use super::prelude::*;

#[derive(Clap)]
/// Find the latest BOOST source
pub struct Boost;

pub fn run(_: Boost, c: Agent) -> VersionResult {
	let url = "https://boostorg.jfrog.io/artifactory/main/release/";
	let h = get(&c, url)?;
	let (_, version, href) = h
		.query("a")
		.filter_map(|a| {
			let mut s = a.inner_text();
			if s.ends_with('/') {
				s.pop();
			}
			let v = Version::parse(&s)?;
			let href = a.href()?;
			Some((v, s, href))
		})
		.max_by(|a, b| a.0.cmp(&b.0))
		.ok_or(NotFound)?;

	version_result(version, Url::parse(url)?.join(&href)?)
}

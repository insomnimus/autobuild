use super::prelude::*;

/// Search GNUPG sources
#[derive(Clap)]
pub struct GnuPg {
	/// Name of the project
	name: String,
	/// File name prefix for a download
	prefix: Option<String>,
}

pub fn run(x: GnuPg, c: Agent) -> VersionResult {
	let mut url = Url::parse(&format!("https://www.gnupg.org/ftp/gcrypt/{}/", x.name))?;
	let prefix = x.prefix.as_deref().unwrap_or(&x.name);

	// GnuTLS special case
	if prefix == "gnutls" {
		let h = get(&c, url.as_str())?;
		let (_, mut dir) = h
			.query("tr")
			.filter_map(|row| {
				let cells = row.query("td").collect::<Vec<_>>();
				if cells.len() > 2 {
					return None;
				}
				let a = cells[1].query("a").next()?;
				let href = a.href()?;
				let s = a.inner_text();
				let v = Version::parse(&s)?;
				Some(((v, Ext::parse(&s)), href))
			})
			.max_by(|a, b| a.0.cmp(&b.0))
			.ok_or(NotFound)?;

		if !dir.ends_with('/') {
			dir.push('/');
		}
		url = url.join(&dir)?;
	}

	let h = get(&c, url.as_str())?;

	let (_, version, href) = h
		.query("tr")
		.filter_map(|row| {
			let cells = row.query("td").collect::<Vec<_>>();
			if cells.len() < 4 {
				return None;
			}

			let a = cells[1].query("a").next()?;
			let s = a.inner_text();
			let (_, s) = extract_version(&s, prefix)?;

			let href = a.href()?;
			let date = cells[2].inner_text();
			if date.is_empty() {
				None
			} else {
				Some((date, s.to_owned(), href))
			}
		})
		.max_by(|a, b| a.0.cmp(&b.0))
		.ok_or(NotFound)?;

	version_result(version, url.join(&href)?)
}

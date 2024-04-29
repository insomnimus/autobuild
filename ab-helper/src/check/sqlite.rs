use headless_chrome::Browser;
use kuchikiki::NodeRef;

use super::prelude::*;

#[derive(Clap)]
/// Find the latest Sqlite source
pub struct Sqlite;

fn get_page() -> Result<NodeRef> {
	info!("instantiating a new headless browser");
	let browser = Browser::default()?;
	info!("opening a new tab on the browser");
	let tab = browser.new_tab()?;
	let url = "https://www.sqlite.org/download.html";
	info!("visiting {url} on the browser");
	tab.navigate_to(url)?;
	tab.wait_until_navigated()?;
	debug!("getting the page HTML");
	let html = tab.get_content()?;
	info!("closing the browser");
	if let Err(e) = tab.close(true) {
		warn!("failed to close the browser; you should kill it manually: {e}");
	}

	Ok(super::html::parse_html(&html))
}

pub fn run(_: Sqlite) -> VersionResult {
	let h = match get_page() {
		Ok(h) => h,
		Err(e) => {
			eprintln!("autobuild warning: failed to check updates using a headless browser: {e}");
			eprintln!("autobuild warning: falling back to a hard-coded version");
			return version_result(
				"3.45.2",
				"https://www.sqlite.org/2024/sqlite-src-3450200.zip",
			);
		}
	};

	let base_url = Url::parse("https://www.sqlite.org").unwrap();
	let download_url = h
		.query("a")
		.find_map(|a| {
			let href = a.href()?;
			let s = a.inner_text();
			if s.starts_with("sqlite-src-") && s.ends_with(".zip") {
				base_url.join(&href).ok()
			} else {
				None
			}
		})
		.ok_or(NotFound)?;

	let re =
		regex::Regex::new(r"(?i)version\s+(v?\d+\.\d+(\.\d+([_+-]?[0-9a-zA-Z_+-]+)?)?)").unwrap();
	// const SUBSTR: &str = "version";
	let version = h
		.query("td")
		.find_map(|td| {
			let s = td.inner_text();
			// s.match_indices(SUBSTR).find_map(|(i, _)| {
			// let s = s[i + SUBSTR.len()..].split_whitespace().next()?;
			// let _ = Version::parse(s)?;
			// Some(s.to_owned())
			// })
			let caps = re.captures(&s)?;
			let _ = Version::parse(&caps[1])?;
			Some(caps[1].to_owned())
		})
		.ok_or(NotFound)?;

	version_result(version, download_url)
}

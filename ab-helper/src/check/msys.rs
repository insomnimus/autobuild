use super::prelude::*;

#[derive(Clap)]
/// Search MSYS2 repositories for a binary package
pub struct Msys {
	/// Name of the package
	name: String,
	/// Find packages linking with the msvcrt C runtime (mingw64)
	#[arg(short, long, group = "crt")]
	msvcrt: bool,
	/// Find Packages linking with the Universal C Runtime (ucrt64)
	#[arg(short, long, group = "crt")]
	ucrt: bool,
}

pub fn run(x: Msys, c: Agent) -> VersionResult {
	let url = if x.ucrt {
		format!(
			"https://packages.msys2.org/package/mingw-w64-ucrt-x86_64-{}?repo=ucrt64",
			x.name
		)
	} else {
		format!(
			"https://packages.msys2.org/package/mingw-w64-x86_64-{}",
			x.name
		)
	};

	let h = get(&c, &url)?;
	let mut version = String::new();
	let mut download_url = String::new();

	for dt in h.query("dt") {
		let s = dt.inner_text();
		match s.as_str() {
			"Version:" => {
				let Some(x) = dt.next_element().filter(|x| x.is("dd")) else {
					continue;
				};

				version = x.inner_text();
				if !version.is_empty() && !download_url.is_empty() {
					break;
				}
			}
			"File:" => {
				let Some(a) = dt.next_element().and_then(|x| x.query("a[href]").next()) else {
					continue;
				};

				let Some(href) = a.href() else {
					continue;
				};

				download_url = href;
				if !version.is_empty() {
					break;
				}
			}
			_ => (),
		}
	}

	if version.is_empty() || download_url.is_empty() {
		Err(NotFound.into())
	} else {
		version_result(version, download_url)
	}
}

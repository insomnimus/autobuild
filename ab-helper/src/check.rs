mod boost;
mod filter;
mod gh;
mod gnome;
mod gnu;
mod gnupg;
mod html;
mod msys;
mod openbsd;
mod prelude;
mod sf;
mod sqlite;
mod vst3sdk;

use std::{
	fmt,
	thread,
	time::Duration,
};

use anyhow::Result;
use clap::Parser as Clap;
use kuchikiki::NodeRef;
use log::{
	error,
	info,
	warn,
};
use ureq::{
	Agent,
	AgentBuilder,
};

use crate::version::Version;

type VersionResult = Result<VersionInfo, anyhow::Error>;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
// Ordered from least preferred to most.
enum Ext {
	Other,
	Lz,
	Lz4,
	Bz,
	Bz2,
	Gz,
	Xz,
	Zst,
}

impl Ext {
	const EXTS: &'static [(&'static str, Self)] = &[
		(".tar.zst", Self::Zst),
		(".tar.zstd", Self::Zst),
		(".tzst", Self::Zst),
		(".tar.xz", Self::Xz),
		(".txz", Self::Xz),
		(".tar.gz", Self::Gz),
		(".tgz", Self::Gz),
		(".tar.bz2", Self::Bz2),
		(".tbz2", Self::Bz2),
		(".tar.bz", Self::Bz),
		(".tbz", Self::Bz),
		(".tar.lz4", Self::Lz4),
		(".tlz4", Self::Lz4),
		(".tar.lz", Self::Lz),
		(".tlz", Self::Lz),
	];

	fn parse(s: &str) -> Self {
		Self::EXTS
			.iter()
			.find_map(|&(ext, val)| if s.ends_with(ext) { Some(val) } else { None })
			.unwrap_or(Self::Other)
	}
}

#[derive(Debug)]
struct NotFound;

impl std::error::Error for NotFound {}
impl fmt::Display for NotFound {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str("couldn't determine a version")
	}
}

struct VersionInfo {
	version: String,
	url: String,
}

fn version_result<V: Into<String>, U: Into<String>>(version: V, url: U) -> VersionResult {
	Ok(VersionInfo {
		version: version.into(),
		url: url.into(),
	})
}

fn get(c: &Agent, url: &str) -> Result<NodeRef> {
	let mut tries = 0;
	loop {
		tries += 1;
		if tries == 1 {
			info!("get {url}");
		} else {
			info!("try #{tries}: get {url}");
		}
		match c.get(url).call() {
			Ok(resp) => return Ok(html::parse_html(&resp.into_string()?)),
			Err(ureq::Error::Status(code, _)) if tries < 5 && code >= 500 => {
				warn!("got status {code}; retrying in 250ms");
				thread::sleep(Duration::from_millis(250))
			}
			Err(e) => {
				error!("error response: {e}");
				return Err(e.into());
			}
		}
	}
}

fn strip_ext(s: &str) -> Option<&str> {
	const EXTS: &[&str] = &[
		".tgz",
		".tar.gz",
		".tbz",
		".tar.bz",
		".tbz2",
		".tar.bz2",
		".txz",
		".tar.xz",
		".tlz",
		".tar.lz",
		".tlz4",
		".tar.lz4",
		".tlz2",
		".tar.lz2",
		".tlzma",
		".tar.lzma",
		".tlzma2",
		".tar.lzma2",
		".tzst",
		".tar.zst",
		".tzstd",
		".tar.zstd",
	];

	for e in EXTS {
		if let Some(s) = s.strip_suffix(e) {
			return Some(s);
		}
	}

	None
}

fn extract_version<'a>(s: &'a str, prefix: &str) -> Option<(Version, &'a str)> {
	let s = strip_ext(s.strip_prefix(prefix)?)?;
	let s = s.strip_prefix('-').unwrap_or(s);
	let s = s.strip_prefix('-').unwrap_or(s);
	Version::parse(s).map(|v| (v, s))
}

/// Check for the latest version and download URL of a project
#[derive(Clap)]
pub struct Check {
	/// If supported, allow pre-release
	#[arg(long)]
	allow_pre: bool,
	#[command(subcommand)]
	cmd: Cmd,
}

#[derive(Clap)]
enum Cmd {
	Boost(boost::Boost),
	Gh(gh::Gh),
	Gnu(gnu::Gnu),
	#[command(name = "gnupg")]
	GnuPg(gnupg::GnuPg),
	Gnome(gnome::Gnome),
	Msys(msys::Msys),
	#[command(name = "openbsd")]
	OpenBsd(openbsd::OpenBsd),
	Sf(sf::Sf),
	Sqlite(sqlite::Sqlite),
	Vst3Sdk(vst3sdk::Vst3Sdk),
}

pub fn run(args: Check) -> Result<()> {
	let c = AgentBuilder::new().user_agent("curl/8.6.0").build();

	let VersionInfo { version, url } = match args.cmd {
		Cmd::Gnu(x) => gnu::run(x, c)?,
		Cmd::Gh(x) => gh::run(x, c)?,
		Cmd::Gnome(x) => gnome::run(x, c)?,
		Cmd::Msys(x) => msys::run(x, c)?,
		Cmd::Sf(x) => sf::run(x, c, args.allow_pre)?,
		Cmd::GnuPg(x) => gnupg::run(x, c)?,
		Cmd::Sqlite(x) => sqlite::run(x)?,
		Cmd::Boost(x) => boost::run(x, c)?,
		Cmd::OpenBsd(x) => openbsd::run(x, c, args.allow_pre)?,
		Cmd::Vst3Sdk(x) => vst3sdk::run(x, c)?,
	};

	println!("{url}\n{version}");
	Ok(())
}

use std::{
	borrow::Cow,
	io::{
		BufRead,
		BufReader,
	},
};

use anyhow::{
	ensure,
	Result,
};
use clap::Parser as Clap;
use serde::Deserialize;
use ureq::AgentBuilder;

use crate::version::Version;

#[derive(Clap)]
/// Get the latest version of a Rust crate from crates.io
pub struct CrateVersion {
	#[arg(value_parser = validate_crate_name)]
	/// Name of the crate
	name: String,
}

fn validate_crate_name(s: &str) -> Result<String, &'static str> {
	if !s.starts_with(|c: char| c.is_ascii_alphabetic())
		|| !s
			.chars()
			.all(|c| c == '_' || c == '-' || c.is_ascii_alphanumeric())
	{
		Err("illegal crate name")
	} else {
		Ok(s.to_string())
	}
}

#[derive(Deserialize)]
struct CrateInfo {
	// name: String,
	#[serde(rename = "vers")]
	version: String,
	#[serde(default)]
	yanked: bool,
}

fn lower(s: &str) -> Cow<str> {
	if s.chars().all(|c| c.is_lowercase()) {
		Cow::Borrowed(s)
	} else {
		Cow::Owned(s.to_lowercase())
	}
}

fn find_latest(crate_name: &str) -> Result<String> {
	let c = AgentBuilder::new()
		.user_agent("autobuild (github.com/insomnimus/autobuild)")
		.build();
	let base = "https://index.crates.io/";
	let mut url = String::with_capacity(base.len() + 7 + crate_name.len());
	url += base;

	match crate_name.len() {
		0 => unreachable!(),
		1 => url.push('1'),
		2 => url.push('2'),
		3 => {
			url += "3/";
			url += &lower(&crate_name[..1]);
		}
		_ => {
			url += &lower(&crate_name[..2]);
			url.push('/');
			url += &lower(&crate_name[2..4]);
		}
	}

	url.push('/');
	url += crate_name;

	let resp = BufReader::new(c.get(&url).call()?.into_reader());

	let mut max = Version::MIN;
	let mut max_str = String::new();
	for res in resp.lines() {
		let CrateInfo { version, yanked } = serde_json::from_str(&res?)?;
		if yanked {
			continue;
		}
		if let Some(ver) = Version::parse(&version) {
			if ver > max {
				max = ver;
				max_str = version;
			}
		}
	}

	ensure!(!max_str.is_empty(), "couldn't find any version");

	Ok(max_str)
}

pub fn run(args: CrateVersion) -> Result<()> {
	println!("{}", find_latest(&args.name)?);
	Ok(())
}

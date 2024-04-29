use std::{
	fmt::Write as FmtWrite,
	fs,
	path::PathBuf,
};

use anyhow::{
	anyhow,
	bail,
	Result,
};
use clap::Parser as Clap;
use log::info;

/// Edit a Libtool .la file
#[derive(Clap)]
pub struct EditLt {
	/// Path to the libtool file
	#[arg(short, long)]
	path: PathBuf,
	/// Change the installation prefix
	#[arg(long, value_parser = validate_relocate)]
	relocate: String,
	/// Print the modified file to stdout instead of saving to disk
	#[arg(long)]
	print: bool,
}

fn validate_relocate(s: &str) -> Result<String> {
	if !s.starts_with('/') {
		bail!("the relocation prefix must be an absolute path");
	}
	if let Some(c) = s
		.matches(|c: char| {
			!c.is_alphanumeric() && !matches!(c, '/' | '-' | '_' | '.' | '+' | '~' | '^' | '%')
		})
		.next()
	{
		bail!("specified prefix contains an illegal character: {c:?}");
	}

	let s = s.trim_end_matches('/');
	Ok(s.to_owned())
}

fn lib_dir_suffix(s: &str) -> &str {
	if s.ends_with("/lib64") {
		"lib64"
	} else if s.ends_with("/lib32") {
		"lib32"
	} else {
		"lib"
	}
}

fn strip_quotes(s: &str) -> Option<&str> {
	s.strip_prefix('\'')?.strip_suffix('\'')
}

pub fn run(args: EditLt) -> Result<()> {
	let contents = fs::read_to_string(&args.path)
		.map_err(|e| anyhow!("failure reading {}: {}", args.path.display(), e))?;
	let mut buf = String::with_capacity(contents.len() + 100);
	for s in contents.lines().map(str::trim) {
		if let Some(val) = s.strip_prefix("libdir=").and_then(strip_quotes) {
			let dir = lib_dir_suffix(val);
			let _ = writeln!(buf, "libdir='{}/{}'", args.relocate, dir);
		} else if let Some(val) = s.strip_prefix("dependency_libs=").and_then(strip_quotes) {
			buf += "dependency_libs='";

			for arg in val.split_whitespace() {
				buf.push(' ');
				if let Some(path) = arg.strip_prefix("-L").filter(|x| !x.is_empty()) {
					let dir = lib_dir_suffix(path);
					let _ = write!(buf, "-L{}/{}", args.relocate, dir);
				} else {
					buf += arg;
				}
			}

			buf += "'\n";
		} else {
			buf += s;
			buf.push('\n');
		}
	}

	if args.print {
		println!("{}", buf.trim());
	} else {
		if let Some(i) = buf.rfind(|c: char| !c.is_whitespace()) {
			buf.truncate(i + 1);
		}
		if contents.trim() == buf.trim_start() {
			info!("no changes");
		} else {
			buf.push('\n');
			fs::write(&args.path, buf.trim_start().as_bytes())
				.map_err(|e| anyhow!("failure saving changes to {}: {}", args.path.display(), e))?;
		}
	}

	Ok(())
}

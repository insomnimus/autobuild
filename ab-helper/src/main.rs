mod check;
mod crate_version;
mod detect_crt;
mod dl;
mod edit_lt;
mod edit_pc;
mod logger;
mod merge_dir;
mod template;
mod toml;
mod version;

use std::env;

use clap::Parser as Clap;
use log::Level;

#[derive(Clap)]
#[command(version)]
/// Autobuild helper utility
struct App {
	/// Enable verbose output; specify twice for debug and 3 times for trace output
	#[arg(short, long, global = true, action = clap::ArgAction::Count)]
	verbose: u8,

	#[command(subcommand)]
	cmd: Cmd,
}

#[derive(Clap)]
enum Cmd {
	Check(check::Check),
	CrateVersion(crate_version::CrateVersion),
	Dl(dl::Dl),
	DetectCrt(detect_crt::DetectCrt),
	EditLt(edit_lt::EditLt),
	EditPc(edit_pc::EditPc),
	MergeDir(merge_dir::MergeDir),
	Toml(toml::Toml),
	/// Print the version string alone
	Version,
}

fn main() {
	fn run() -> anyhow::Result<()> {
		let args = App::parse();

		logger::init(match args.verbose {
			0 => env::var("AB_HELPER_VERBOSE").ok().and_then(|mut s| {
				s.make_ascii_lowercase();
				match s.as_str() {
					"" | "0" | "off" => None,
					"2" | "debug" => Some(Level::Debug),
					"3" | "trace" => Some(Level::Trace),
					_ => Some(Level::Info),
				}
			}),
			1 => Some(Level::Info),
			2 => Some(Level::Debug),
			_ => Some(Level::Trace),
		});

		match args.cmd {
			Cmd::Version => {
				println!("{}", env!("CARGO_PKG_VERSION"));
				Ok(())
			}
			Cmd::Check(x) => check::run(x),
			Cmd::CrateVersion(x) => crate_version::run(x),
			Cmd::Dl(x) => dl::run(x),
			Cmd::DetectCrt(x) => detect_crt::run(x),
			Cmd::EditPc(x) => edit_pc::run(x),
			Cmd::EditLt(x) => edit_lt::run(x),
			Cmd::MergeDir(x) => merge_dir::run(x),
			Cmd::Toml(x) => toml::run(x),
		}?;

		Ok(())
	}

	if let Err(e) = run() {
		eprintln!("error: {e}");
		std::process::exit(1);
	}
}

use std::{
	fs::{
		self,
		File,
	},
	io::Read,
	path::PathBuf,
	process::exit,
};

use anyhow::{
	anyhow,
	ensure,
	Result,
};
use clap::Parser as Clap;
use goblin::pe::PE;

#[derive(Clap)]
/// Detect the dynamically linked C runtime of a PE binary
pub struct DetectCrt {
	/// Path to the PE binary
	path: PathBuf,
	/// Exit with exit code 1 if the binary does not link to the universal C runtime (ucrt)
	#[arg(short = 'u', long, group = "action")]
	is_ucrt: bool,
	/// Exit with exit code 1 if the binary does not link to msvcrt
	#[arg(short = 'm', long, group = "action")]
	is_msvcrt: bool,
	/// Print the list of DLL's being imported
	#[arg(short, long, group = "action")]
	list: bool,
}

pub fn run(args: DetectCrt) -> Result<()> {
	let md = fs::metadata(&args.path).map_err(|e| anyhow!("failed to read file: {e}"))?;
	ensure!(md.len() > 64, "not a PE binary");
	ensure!(
		md.file_type().is_file(),
		"not a file: {}",
		args.path.display()
	);

	let mut f = File::open(&args.path).map_err(|e| anyhow!("failed to read file: {e}"))?;

	let mut magic = [0; 2];
	f.read_exact(&mut magic)?;

	let n = u16::from_be_bytes(magic);
	ensure!(n == 0x4d5a, "not a PE binary");

	let mut data = Vec::with_capacity(
		md.len()
			.try_into()
			.map_err(|_| anyhow!("the file is too big"))?,
	);
	data.extend(magic);
	f.read_to_end(&mut data)?;
	drop(f);

	let pe = PE::parse(&data).map_err(|_| anyhow!("not a valid PE binary"))?;
	if args.list {
		for s in &pe.libraries {
			println!("{s}");
		}
		return Ok(());
	}

	for s in pe.libraries {
		if s == "msvcrt.dll" {
			if args.is_ucrt {
				exit(1);
			}
			if !args.is_msvcrt {
				println!("msvcrt");
			}
			return Ok(());
		}

		if s.starts_with("api-ms-win-crt") {
			if args.is_msvcrt {
				exit(1);
			}
			if !args.is_ucrt {
				println!("ucrt");
			}
			return Ok(());
		}
	}

	Err(anyhow!(
		"couldn't detect any dynamically linked C runtime; it's probably statically linked"
	))
}

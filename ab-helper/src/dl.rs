use std::{
	fs::{
		self,
		File,
	},
	io,
	io::Read,
	path::{
		Path,
		PathBuf,
	},
	thread,
	time::Duration,
};

use anyhow::{
	anyhow,
	bail,
	ensure,
	Result,
};
use clap::Parser as Clap;
use log::{
	error,
	info,
	warn,
};
use ureq::{
	Agent,
	AgentBuilder,
	Response,
};
use url::Url;

/// Download a file from the internet
#[derive(Clap)]
pub struct Dl {
	/// The download URL
	#[arg(value_parser = Url::parse)]
	url: Url,
	/// Save the downloaded file to another path (use - for stdout)
	#[arg(short, long, short_alias = 'O')]
	out: PathBuf,

	/// Maximum number of retries in case the server responds with a status code >= 500
	#[arg(short = 'r', long, default_value_t = 5)]
	max_retries: u16,
	/// Maximum size of a download; if more bytes are downloaded then the download aborts (useful to avoid malicious servers)
	///
	/// You can use a size suffix: b, kb, mb, gb
	#[arg(long, default_value = "4gb", value_parser = parse_size)]
	max_size: u64,

	/// Ignored; kept for compatibility with wget
	#[arg(short = 'q', long = "quiet")]
	_quiet: bool,
}

fn parse_size(s: &str) -> Result<u64> {
	ensure!(!s.starts_with('-'), "value cannot be negative");
	let s = s.to_lowercase();
	let s = s
		.strip_suffix("ib")
		.or_else(|| s.strip_suffix('b'))
		.unwrap_or(&s);

	let c = s
		.get(s.len() - 1..)
		.ok_or_else(|| anyhow!("invalid size specifier"))?;
	// Now we can safely slice the string as we didn't butcher a unicode multibyte sequence above.
	let mut val = &s[..s.len() - 1];
	let unit: u64 = match c {
		"t" => 1 << 40,
		"g" => 1 << 30,
		"m" => 1 << 20,
		"k" => 1 << 10,
		"b" => 1,
		_ => {
			val = s;
			1
		}
	};

	// Attempt at covering the edge case of user providing an integer greater than 1 >> 52.
	if !val.contains('.') {
		return Ok(val.parse::<u64>()?.saturating_mul(unit));
	}

	let n = val.parse::<f64>().map_err(|_| {
		anyhow!("value does not appear to contain a valid number or a known size suffix")
	})?;

	ensure!(!n.is_nan(), "value cannot be NaN");

	// If it's larger than 1 << 52, we just saturate there.
	if n >= (1_u64 << 52) as f64 {
		return Ok(1 << 52);
	}

	let n = f64::min(n * unit as f64, (1_u64 << 52) as f64);
	if n.is_infinite() {
		Ok(1 << 52)
	} else if n <= 1.01 {
		Ok(1)
	} else {
		Ok(n as u64)
	}
}

fn get_retry(c: &Agent, url: &str, max_retries: u16) -> Result<Response> {
	let mut retries = 0;
	loop {
		if retries == 0 {
			info!("get {url}");
		} else {
			info!("retry #{retries}: get {url}");
		}

		match c.get(url).call() {
			Ok(resp) => return Ok(resp),
			Err(ureq::Error::Status(code, _)) if retries < max_retries && code >= 500 => {
				// Double the sleep duration every $retries, starting at 250ms.
				// Capped at 512 seconds.
				let sleep_ms = 250_u64 << u16::min(retries, 11);
				let secs = sleep_ms as f32 / 1000.0;
				warn!("got status {code}; retrying in {secs:.2}s");
				thread::sleep(Duration::from_millis(sleep_ms));
			}
			Err(e) => return Err(e.into()),
		}

		retries += 1;
	}
}

fn temp_name(p: &Path) -> PathBuf {
	use rand::{
		distributions::Alphanumeric,
		Rng,
	};

	let parent = p.parent().unwrap_or(Path::new("."));
	let mut rng = rand::thread_rng();
	let mut name = String::with_capacity(32);
	name += "_ab_helper_dl_";
	name.extend((0_u8..8).map(|_| rng.sample(Alphanumeric) as char));
	name += ".tmp";
	parent.join(name)
}

pub fn run(args: Dl) -> Result<()> {
	let c = AgentBuilder::new().user_agent("Wget/1.24.5").build();
	let resp = get_retry(&c, args.url.as_str(), args.max_retries)?;
	let max = if args.max_size == 0 {
		u64::MAX - 1
	} else {
		args.max_size
	};

	// We Limit it to 1 more than args.max_size so we can check if we over-read and fail if so.
	let mut body = resp.into_reader().take(max + 1);

	if args.out.as_os_str() == "-" {
		let mut stdout = io::stdout().lock();
		let read = io::copy(&mut body, &mut stdout)?;
		if args.max_size != 0 && read > args.max_size {
			bail!("exceeded maximum file size limit of {max} bytes");
		}
		return Ok(());
	}

	let tmp = temp_name(&args.out);
	let mut f = File::create(&tmp)
		.map_err(|e| anyhow!("failure creating file {}: {}", tmp.display(), e))?;

	match io::copy(&mut body, &mut f) {
		Err(e) => {
			eprintln!("error: failed to copy response body: {e}");
			warn!("removing temporary file {}", tmp.display());
			drop(f);
			if let Err(e) = fs::remove_file(&tmp) {
				error!("failed to remove temporary file {}: {}", tmp.display(), e);
			}
			bail!("download failed; see previous messages");
		}
		Ok(read) if args.max_size != 0 && read > args.max_size => {
			eprintln!("error: exceeded maximum file size limit of {max} bytes");
			warn!("removing temporary file");
			drop(f);
			if let Err(e) = fs::remove_file(&tmp) {
				error!("failed to remove temporary file {}: {}", tmp.display(), e);
			}
			bail!("maximum download size exceeded");
		}
		Ok(_) => (),
	}

	drop(f);
	info!("renaming temporary file");
	fs::rename(&tmp, &args.out).map_err(|e| {
		anyhow!(
			"failure renaming temporary file {} -> {}: {}",
			tmp.display(),
			args.out.display(),
			e
		)
	})
}

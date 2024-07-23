use std::{
	collections::BTreeSet,
	fs,
	io::ErrorKind,
	path::{
		Path,
		PathBuf,
	},
	sync::{
		Arc,
		Mutex,
	},
};

use anyhow::{
	anyhow,
	bail,
	Result,
};
use clap::Parser as Clap;
use globset::{
	GlobBuilder,
	GlobSet,
	GlobSetBuilder,
};
use jwalk::{
	rayon::prelude::*,
	WalkDir,
};
use log::{
	debug,
	info,
	log_enabled,
};

/// Merge contents of one directory into another
#[derive(Clap)]
pub struct MergeDir {
	/// The directory containing files to be copied
	source: PathBuf,
	/// The directory the files will be copied into
	dest: PathBuf,

	/// Ignore file names matching the specified glob pattern (can be specified multiple times)
	#[arg(short, long)]
	exclude: Vec<String>,
	/// Only select file names matching the specified glob pattern (can be specified multiple times)
	#[arg(short, long)]
	glob: Vec<String>,
}

#[derive(Clone)]
struct Glob {
	gs: Arc<GlobSet>,
}

impl Glob {
	fn new(globs: &[String]) -> Result<Self, globset::Error> {
		let mut b = GlobSetBuilder::new();
		for g in globs {
			b.add(
				GlobBuilder::new(g)
				.case_insensitive(cfg!(windows))
				// We don't care about slashes since it's matched against file names only.
				.literal_separator(false)
				.backslash_escape(cfg!(not(windows)))
				.empty_alternates(true)
				.build()?,
			);
		}

		Ok(Self {
			gs: Arc::new(b.build()?),
		})
	}

	// This is different than !is_match because if self.gs has no globs, this returns `false` instead of `true`.
	fn is_mismatch<P: AsRef<Path>>(&self, s: P) -> bool {
		!self.gs.is_empty() && !self.gs.is_match(s.as_ref())
	}

	fn is_match<P: AsRef<Path>>(&self, s: P) -> bool {
		self.gs.is_match(s.as_ref())
	}
}

pub fn run(args: MergeDir) -> Result<()> {
	let exclude = Glob::new(&args.exclude).map_err(|e| anyhow!("glob error: {e}"))?;
	let include = Glob::new(&args.glob).map_err(|e| anyhow!("glob error: {e}"))?;

	// Use the canonicalized source path for iteration.
	let src = args.source.canonicalize().map_err(|e| {
		anyhow!(
			"failed to canonicalize source directory {}: {}",
			args.source.display(),
			e
		)
	})?;

	match fs::metadata(&src) {
		Err(e) => bail!("failed to read directory {}: {}", args.source.display(), e),
		Ok(md) if !md.is_dir() => bail!("source is not a directory ({})", args.source.display()),
		_ => (),
	};

	let walker = WalkDir::new(&src)
		.skip_hidden(false)
		.follow_links(false)
		.min_depth(1)
		.process_read_dir({
			let include = Glob::clone(&include);
			let exclude = Glob::clone(&exclude);

			move |_, path, _, entries| {
				entries.retain(|entry| match entry {
					Err(_) => true,
					Ok(entry) => {
						if !exclude.is_match(&entry.file_name)
							&& !include.is_mismatch(&entry.file_name)
						{
							true
						} else {
							debug!("ignored {}", path.display());
							false
						}
					}
				});
			}
		});

	// Ensure dest exists
	match fs::create_dir(&args.dest) {
		Ok(_) => (),
		Err(e) if e.kind() == ErrorKind::AlreadyExists => (),
		Err(e) => bail!(
			"error creating destination directory {}: {}",
			args.dest.display(),
			e
		),
	}

	let dirs_created = Arc::new(Mutex::new(BTreeSet::new()));

	walker
		.try_into_iter()?
		.par_bridge()
		.map_with(
			(PathBuf::with_capacity(256), PathBuf::with_capacity(256)),
			{
				let dirs_created = Arc::clone(&dirs_created);
				move |&mut (ref mut src_buf, ref mut dest_buf), entry| {
					let entry = entry?;

					dest_buf.clear();
					dest_buf.push(&args.dest);
					dest_buf.push(entry.parent_path.strip_prefix(&src).unwrap());
					dest_buf.push(&entry.file_name);

					if exclude.is_match(&entry.file_name) || include.is_mismatch(&entry.file_name) {
						debug!("ignored {}", dest_buf.display());
						return anyhow::Ok(());
					}

					if entry.file_type().is_dir() {
						if log_enabled!(log::Level::Info) && !dest_buf.exists() {
							info!("create directory {}", dest_buf.display());
						}
						fs::create_dir_all(&dest_buf).map_err(|e| {
							anyhow!("error creating directory {}: {}", dest_buf.display(), e)
						})?;
						dirs_created
							.lock()
							.unwrap()
							.insert(Arc::<Path>::from(dest_buf.clone()));
						return Ok(());
					}

					// It's a file.
					if dirs_created
						.lock()
						.unwrap()
						.insert(Arc::clone(&entry.parent_path))
					{
						let dest_parent = dest_buf.parent().unwrap();
						fs::create_dir_all(dest_parent).map_err(|e| {
							anyhow!("error creating directory {}: {}", dest_parent.display(), e)
						})?;
					}

					// Is dest a directory?
					match fs::symlink_metadata(&dest_buf) {
						Err(e) if e.kind() == ErrorKind::NotFound => (),
						Ok(md) if md.file_type().is_dir() => {
							debug!("remove directory {}", dest_buf.display());
							fs::remove_dir_all(&dest_buf).map_err(|e| {
								anyhow!("failed to remove directory {}: {}", dest_buf.display(), e)
							})?;
						}
						_ => (),
					}

					src_buf.clear();
					src_buf.push(&entry.parent_path);
					src_buf.push(&entry.file_name);
					info!("copy {} to {}", src_buf.display(), dest_buf.display());
					fs::copy(&src_buf, &dest_buf).map_err(|e| {
						anyhow!(
							"failed to copy {} to {}: {}",
							src_buf.display(),
							dest_buf.display(),
							e
						)
					})?;

					Ok(())
				}
			},
		)
		.collect::<Result<(), _>>()?;

	Ok(())
}

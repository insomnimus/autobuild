use std::{
	collections::BTreeSet,
	fs::{
		self,
		Permissions,
	},
	io::{
		self,
		ErrorKind,
	},
	path::{
		Path,
		PathBuf,
	},
	sync::Arc,
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
use jwalk::WalkDir;
use log::{
	debug,
	info,
	trace,
};

/// Merge contents of one directory into another
#[derive(Clap)]
pub struct MergeDir {
	/// The directory containing files to be copied
	source: PathBuf,
	/// The directory the files will be copied into
	dest: PathBuf,

	/// Ignore names matching the specified glob pattern (can be specified multiple times)
	#[arg(short, long)]
	exclude: Vec<String>,
	/// Only select names matching the specified glob pattern (can be specified multiple times)
	#[arg(short, long)]
	glob: Vec<String>,
}

#[derive(Clone)]
struct Glob {
	gs: GlobSet,
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

		Ok(Self { gs: b.build()? })
	}

	// This is different than !is_match because if self.gs has no globs, this returns `false` instead of `true`.
	fn is_mismatch<P: AsRef<Path>>(&self, s: P) -> bool {
		!self.gs.is_empty() && !self.gs.is_match(s.as_ref())
	}

	fn is_match<P: AsRef<Path>>(&self, s: P) -> bool {
		self.gs.is_match(s.as_ref())
	}
}

fn remove_readonly<P: AsRef<Path>>(p: P, mut perms: Permissions) -> io::Result<()> {
	let p = p.as_ref();
	trace!("remove read-only permission from {}", p.display());

	#[cfg(not(unix))]
	{
		#[allow(clippy::permissions_set_readonly_false)]
		perms.set_readonly(false);
	}

	#[cfg(unix)]
	{
		use std::os::unix::fs::PermissionsExt;
		let mode = perms.mode();
		// Set the user-write bit
		perms.set_mode(mode | 0o200);
	}

	fs::set_permissions(p, perms)
}

fn remove_file<P: AsRef<Path>>(p: P) -> Result<()> {
	let p = p.as_ref();
	debug!("remove file {}", p.display());
	fs::remove_file(p).map_err(|e| anyhow!("failed to remove file {}: {}", p.display(), e))
}

fn create_dir_all<P: AsRef<Path>>(p: P) -> Result<()> {
	let p = p.as_ref();
	debug!("create directory {}", p.display());
	fs::create_dir_all(p).map_err(|e| anyhow!("error creating directory {}: {}", p.display(), e))
}

pub fn run(args: MergeDir) -> Result<()> {
	let exclude = Arc::new(Glob::new(&args.exclude).map_err(|e| anyhow!("glob error: {e}"))?);
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
			let exclude = Arc::clone(&exclude);

			move |_, path, _, entries| {
				entries.retain(|entry| match entry {
					Err(_) => true,
					Ok(entry) => {
						if exclude.is_match(&entry.file_name) {
							debug!("ignored {}", path.display());
							false
						} else {
							true
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

	// The source directories that we know exist on the destination.
	let mut src_dirs = BTreeSet::<Arc<Path>>::new();
	// These buffers are reused to avoid allocations.
	let mut src_buf = PathBuf::with_capacity(256);
	let mut dest_buf = PathBuf::with_capacity(256);

	for entry in walker.try_into_iter()? {
		let entry = entry?;

		dest_buf.clear();
		dest_buf.push(&args.dest);
		dest_buf.push(entry.parent_path.strip_prefix(&src).unwrap());
		dest_buf.push(&entry.file_name);

		if exclude.is_match(&entry.file_name) || include.is_mismatch(&entry.file_name) {
			debug!("ignored {}", dest_buf.display());
			continue;
		}

		src_buf.clear();
		src_buf.push(&entry.parent_path);
		src_buf.push(&entry.file_name);

		if entry.file_type().is_dir() {
			if src_dirs.contains(src_buf.as_path()) {
				continue;
			}

			match fs::metadata(&dest_buf) {
				// If dest is a file, remove it.
				Ok(md) if md.file_type().is_file() => {
					let perms = md.permissions();
					if perms.readonly() {
						remove_readonly(&dest_buf, perms).map_err(|e| {
							anyhow!(
								"error removing the readonly permission from {}: {}",
								dest_buf.display(),
								e
							)
						})?;
					}
					remove_file(&dest_buf)?;
				}
				// If it's already a directory, there's nothing to do.
				Ok(md) if md.file_type().is_dir() => {
					src_dirs.insert(Arc::from(src_buf.as_path()));
					continue;
				}
				// If it's an error, proceed to attempt creating it anyway.
				_ => (),
			}

			create_dir_all(&dest_buf)?;
			src_dirs.insert(Arc::<Path>::from(dest_buf.clone()));
			continue;
		}

		// It's a file.
		if src_dirs.insert(Arc::clone(&entry.parent_path)) {
			let dest_parent = dest_buf.parent().unwrap();
			create_dir_all(dest_parent)?;
		}

		// Remove destination if it exists.
		match fs::symlink_metadata(&dest_buf) {
			Err(e) if e.kind() == ErrorKind::NotFound => (),
			Ok(md) if md.file_type().is_dir() => {
				debug!("remove directory {}", dest_buf.display());
				fs::remove_dir_all(&dest_buf).map_err(|e| {
					anyhow!("failed to remove directory {}: {}", dest_buf.display(), e)
				})?;
			}
			Ok(md) => {
				if md.permissions().readonly() {
					remove_readonly(&dest_buf, md.permissions()).map_err(|e| {
						anyhow!(
							"error removing the readonly permission from {}: {}",
							dest_buf.display(),
							e
						)
					})?;
				}

				remove_file(&dest_buf)?;
			}
			_ => (),
		}

		info!("copy {} to {}", src_buf.display(), dest_buf.display());
		fs::copy(&src_buf, &dest_buf).map_err(|e| {
			anyhow!(
				"failed to copy {} to {}: {}",
				src_buf.display(),
				dest_buf.display(),
				e
			)
		})?;
	}

	Ok(())
}

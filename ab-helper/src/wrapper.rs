use std::{
	borrow::{
		Cow,
		Cow::*,
	},
	env,
	ffi::{
		OsStr,
		OsString,
	},
	process::Command,
};

const ENV_CALLING_FROM: &str = "_AB_HELPER_CALLING_FROM_WRAPPER";

macro_rules! exit_err {
	[$($args:tt)+] => {{
		eprintln!($($args)+);
		std::process::exit(1);
	}};
}

fn check_recursion(wrapper: &str) {
	if env::var_os(ENV_CALLING_FROM).is_some_and(|s| s == wrapper) {
		exit_err!("{wrapper}: error: wrapper recursion detected");
	}
}

fn cow_str(s: &str) -> Cow<OsStr> {
	Borrowed(OsStr::new(s))
}

#[inline]
fn is_sys_path(s: &OsStr) -> bool {
	s == "-I/usr/include" || s == "-L/usr/lib" || s == "-L/lib" || s == "-L/lib/"
}

fn exec<I>(wrapper_name: &str, prog: &str, args: I) -> !
where
	I: IntoIterator,
	I::Item: AsRef<OsStr>,
{
	#[cfg(unix)]
	{
		use std::os::unix::process::CommandExt;
		let e = Command::new(prog)
			.args(args)
			.env(ENV_CALLING_FROM, wrapper_name)
			.exec();
		exit_err!("{wrapper_name}: error: {e}");
	}

	#[cfg(not(unix))]
	match Command::new(prog)
		.args(args)
		.env(ENV_CALLING_FROM, wrapper_name)
		.status()
	{
		Err(e) => {
			exit_err!("{wrapper_name}: error: {e}");
		}
		Ok(status) if !status.success() => {
			std::process::exit(status.code().unwrap_or(1));
		}
		_ => std::process::exit(0),
	}
}

pub fn exec_clang<I>(argv: I) -> !
where
	I: IntoIterator<Item = OsString>,
{
	let ignore_sys_paths = env::var_os("AB_IGNORE_SYSTEM_PATHS").is_some_and(|s| &s == "1");
	let mut args = Vec::with_capacity(64);

	args.extend(["--target=x86_64-w64-mingw32", "-fuse-ld=lld"].map(cow_str));

	let cflags_before = env::var("AB_CFLAGS_BEFORE");
	if let Ok(cflags) = &cflags_before {
		args.extend(cflags.split_whitespace().map(cow_str));
	}

	if let Some(sysroot) = env::var_os("AB_MINGW_SYSROOT") {
		args.push(cow_str("--sysroot"));
		args.push(Owned(sysroot));
	}

	for a in argv {
		// Some projects hardcode -O3 but it's got little to no gains.
		if &a == "-O3" {
			let s = format!(
				"-O{}",
				env::var("AB_OPT_LEVEL").ok().as_deref().unwrap_or("2")
			);
			args.push(Owned(OsString::from(s)));
		} else if !ignore_sys_paths && is_sys_path(&a) {
			exit_err!(
				"ab-clang: error: attempted to use system's native libraries: {}",
				a.to_string_lossy()
			);
		} else {
			args.push(Owned(a));
		}
	}

	let cflags_after = env::var("AB_CFLAGS");
	if let Ok(cflags) = &cflags_after {
		args.extend(cflags.split_whitespace().map(cow_str));
	}

	exec("ab-clang", "clang", args);
}

pub fn exec_clangxx<I>(argv: I) -> !
where
	I: IntoIterator<Item = OsString>,
{
	let ignore_sys_paths = env::var_os("AB_IGNORE_SYSTEM_PATHS").is_some_and(|s| &s == "1");
	let mut args = Vec::with_capacity(64);

	args.extend(["--target=x86_64-w64-mingw32", "-fuse-ld=lld"].map(cow_str));

	let cxxflags_before = env::var("AB_CXXFLAGS_BEFORE");
	if let Ok(flags) = &cxxflags_before {
		args.extend(flags.split_whitespace().map(cow_str));
	}

	if let Some(sysroot) = env::var_os("AB_MINGW_SYSROOT") {
		args.push(cow_str("--sysroot"));
		args.push(Owned(sysroot));
	}

	for a in argv {
		// Some projects hardcode -O3 but it's got little to no gains.
		if &a == "-O3" {
			let s = format!(
				"-O{}",
				env::var("AB_OPT_LEVEL").ok().as_deref().unwrap_or("2")
			);
			args.push(Owned(OsString::from(s)));
		} else if !ignore_sys_paths && is_sys_path(&a) {
			exit_err!(
				"ab-clang++: error: attempted to use system's native libraries: {}",
				a.to_string_lossy()
			);
		} else {
			args.push(Owned(a));
		}
	}

	let cxxflags_after = env::var("AB_CXXFLAGS");
	if let Ok(flags) = &cxxflags_after {
		args.extend(flags.split_whitespace().map(cow_str));
	}

	exec("ab-clang++", "clang++", args);
}

pub fn exec_lld<I>(argv: I) -> !
where
	I: IntoIterator<Item = OsString>,
{
	// This wrapper exists because some projects like to pass -E to lld, which lld doesn't understand.

	let argv = argv.into_iter().filter(|a| a != "-E");

	if let Ok(flags) = env::var("AB_LDFLAGS_AFTER") {
		let mut args = Vec::with_capacity(64);
		args.extend(argv.map(Cow::Owned));
		args.extend(flags.split_whitespace().map(cow_str));
		exec("ab-lld", "ld.lld", args);
	} else {
		exec("ab-lld", "ld.lld", argv);
	}
}

pub fn exec_gcc<I>(argv: I) -> !
where
	I: IntoIterator<Item = OsString>,
{
	check_recursion("ab-g++");

	let ab_cc = env::var("AB_CC").ok();
	match ab_cc.as_deref().unwrap_or("x86_64-w64-mingw32-gcc") {
		"ab-clang" => exec_clang(argv),
		cmd => {
			let ignore_sys_paths = env::var_os("AB_IGNORE_SYSTEM_PATHS").is_some_and(|s| &s == "1");
			let mut args = Vec::with_capacity(64);

			let cflags_before = env::var("AB_CFLAGS_BEFORE");
			if let Ok(cflags) = &cflags_before {
				args.extend(cflags.split_whitespace().map(cow_str));
			}

			for a in argv {
				// Some projects hardcode -O3 but it's got little to no gains.
				if &a == "-O3" {
					let s = format!(
						"-O{}",
						env::var("AB_OPT_LEVEL").ok().as_deref().unwrap_or("2")
					);
					args.push(Owned(OsString::from(s)));
				} else if !ignore_sys_paths && is_sys_path(&a) {
					exit_err!(
						"ab-gcc: error: attempted to use system's native libraries: {}",
						a.to_string_lossy()
					);
				} else {
					args.push(Owned(a));
				}
			}

			let cflags_after = env::var("AB_CFLAGS");
			if let Ok(cflags) = &cflags_after {
				args.extend(cflags.split_whitespace().map(cow_str));
			}

			exec("ab-gcc", cmd, args);
		}
	}
}

pub fn exec_gxx<I>(argv: I) -> !
where
	I: IntoIterator<Item = OsString>,
{
	check_recursion("ab-g++");

	let ab_cxx = env::var("AB_CXX").ok();
	match ab_cxx.as_deref().unwrap_or("x86_64-w64-mingw32-g++") {
		"ab-clang" => exec_clangxx(argv),
		cmd => {
			let ignore_sys_paths = env::var_os("AB_IGNORE_SYSTEM_PATHS").is_some_and(|s| &s == "1");
			let mut args = Vec::with_capacity(64);

			let cxxflags_before = env::var("AB_CXXFLAGS_BEFORE");
			if let Ok(flags) = &cxxflags_before {
				args.extend(flags.split_whitespace().map(cow_str));
			}

			for a in argv {
				// Some projects hardcode -O3 but it's got little to no gains.
				if &a == "-O3" {
					let s = format!(
						"-O{}",
						env::var("AB_OPT_LEVEL").ok().as_deref().unwrap_or("2")
					);
					args.push(Owned(OsString::from(s)));
				} else if !ignore_sys_paths && is_sys_path(&a) {
					exit_err!(
						"ab-g++: error: attempted to use system's native libraries: {}",
						a.to_string_lossy()
					);
				} else {
					args.push(Owned(a));
				}
			}

			let cxxflags_after = env::var("AB_CXXFLAGS");
			if let Ok(flags) = &cxxflags_after {
				args.extend(flags.split_whitespace().map(cow_str));
			}

			exec("ab-g++", cmd, args);
		}
	}
}

pub fn exec_ld<I>(argv: I) -> !
where
	I: IntoIterator<Item = OsString>,
{
	check_recursion("ab-ld");
	let ab_ld = env::var("AB_LD").ok();
	match ab_ld.as_deref().unwrap_or("x86_64-w64-mingw32-ld") {
		"ab-lld" => exec_lld(argv),
		"ab-ld" => exit_err!("ab-ld: error: $AB_LD is set to ab-ld, causing infinite recursion"),
		cmd => {
			let argv = argv.into_iter().filter(|a| a != "-E");

			if let Ok(flags) = env::var("AB_LDFLAGS_AFTER") {
				let mut args = Vec::with_capacity(64);
				args.extend(argv.map(Cow::Owned));
				args.extend(flags.split_whitespace().map(cow_str));
				exec("ab-ld", cmd, args);
			} else {
				exec("ab-ld", cmd, argv);
			}
		}
	}
}

pub fn exec_pkg_config<I>(argv: I) -> !
where
	I: IntoIterator<Item = OsString>,
{
	let pc_libdir = [
		env::var_os("PKG_CONFIG_PATH"),
		env::var_os("PKG_CONFIG_LIBDIR"),
		env::var_os("AB_MINGW_SYSROOT")
			.filter(|s| !s.is_empty())
			.map(|mut s| {
				let path = s.clone();
				s.push("/x86_64-w64-mingw32/lib/pkgconfig:");
				s.push(path);
				s.push("/x86_64-w64-mingw32/share/pkgconfig");
				s
			}),
	]
	.into_iter()
	.flatten()
	.reduce(|mut buf, s| {
		buf.push(":");
		buf.push(s);
		buf
	});

	if let Some(pc_libdir) = &pc_libdir {
		env::set_var("PKG_CONFIG_LIBDIR", pc_libdir);
		env::set_var("PKG_CONFIG_PATH", pc_libdir);
	}

	exec("ab-pkg-config", "pkg-config", argv);
}

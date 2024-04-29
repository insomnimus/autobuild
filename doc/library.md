# Library Functions
There are functions to abstract away the boiler plate of fetching source code, checking for updates and building programs.
They're listed below.

# Conventions
When a parameter description contains "(filtered)", that means that the argument(s) are filtered.
The filters take the form `[app:|lib:][llvm:|gcc:]<value>`.

- If an app is being built, `lib:` values will be ignored; if a library is being built, `app:` ones are ignored.
- If `llvm` or `gcc` is provided and if `$AB_TOOLCHAIN` does not match the toolchain provided, the argument will be ignored.

For example:
- `app:--enable-programs` will let the `--build-programs` flag pass through if an app is being built.
- `llvm:-DFOO=1` will pass the `-DFOO=1` option if the current toolchain (`$AB_TOOLCHAIN`) is llvm.
- `lib:gcc:--banana` will only pass the flag `--banana` if the toolchain's gcc and if a library is being built.

The `app` and `lib` filters are pretty much only relevant for "multi recipes", in other words, recipes that provide a library for other recipes as well as a program for the user.

## `eprint`
Parameters:
- Message to be printed

Prints to stderr.

## `error`
Parameters:
- Message to be printed

Prints a message and exits the script.
It's used for internal bug detectionp rather than general purpose error printing. If it's invoked, it should mean there's a bug in the program.

## `info`, `verbose`
Parameters:
- Any number of messages

These functions write logs.  `verbose` writes only if the user provides the `--verbose` option to `autobuild.sh`.

## `run`, `run_verbose`
Parameters:
- A command and its arguments

These functions print the command, before executing it.
`run_verbose` will only print the command if the user specified the `--verbose` option to `autobuild.sh`.

## `download`
Parameters:
- Recipe name
- (Mandatory for libraries) a path to test for, to determine if it's installed or not
- Optionally any number of extra arguments to pass to more specific download functions that are used privately.

This function prepares the source code: it ensures that it's checked for updates  and downloaded.
It also makes sure to exit the recipe if there is no need to build.

It's not required for app recipes; the driver script calls it before executing an app recipe.
However "multi" recipes will still need to call it for version detection to work properly.

The download source is set in `repos.sh` per recipe.
It will change the script's working directory into the root of the source tree.

## `out`
Parameters:
- One or more paths to be included into the archive

Used to provide the output of application recipes; it will copy the provided files and folders into a temporary directory and create an archive in the `$AB_ROOT/install/$RECIPE`.

If the calling recipe is not currently building an app, this function will do nothing.
So if a "multi" recipe is invoked by another recipe to install its library, calling `out` has no effect.

## `install_lib`
Parameters:
- Any number of recipe names (filtered)

`install_lib` installs libraries into `$AB_PREFIX`.

## `run_configure`
Parameters:
- Path to a configure script
- Arguments to pass to the script (filtered)

Runs an Autotools configure script, pre-populating some common flags such as `--enable-static`, `--host`, `--prefix` etc.

It will check for the configure script; if it's not found it will check for the existence of `./bootstrap`, `./bootstrap.sh`, `./autogen` and `./autogen.sh`; if none of those are found, it will execute `autoreconf -fi`.

By default the script is ran using `dash`, which can break non-posix compliant configure scripts. Set the `$USE_DASH` variable to `0` to run the script without `dash`.

## `configure_install`, `configure_install_par`, `configure_build`, `configure_build_par`
Parameters:
- Arguments for the configure script (filtered)

Runs configure using `run_configure`, passing specified arguments to it.
After that, the `_build` variant builds the project; the `_install` variant additionally performs a `make install`.

The `_par` variants invoke `make` with parallel jobs set to `$JOBS`. Since some projects do not account for parallel Makefiles, it's provided as a separate option.

## `run_cmake`, `cmake_build` and `cmake_install`
Parameters:
- Directory containing `CMakeLists.txt`
- Arguments to pass to cmake (filtered)

`run_cmake`Invokes `cmake` to configure the build with some common flags such as `-DCMAKE_INSTALL_PREFIX, `-DCMAKE_TOOLCHAIN_FILE` and `CMAKE_BUILD_TYPE` pre-populated.

`cmake_build` invokes `run_cmake` and then builds the project.

`cmake_install` additionally installs the project to `$AB_PREFIX`.

# `meson_build` and `meson_install`
Parameters:
- Arguments to pass to `meson setup` (filtered)

Configures a meson based project with some common options pre-populated, then starts the build.
The build will be performed in the `autobuild` directory under the script's working directory.

`meson_install` will additionally install into `$AB_PREFIX`.

## `cargo_build`
Parameters:
- Arguments to be forwarded to `cargo`

Builds a cargo based Rust application.
It will set common flags such as `--target` and `--release`.
It also sets some environment variables used by common Rust build scripts such as `TARGET_CC`, transferring the `CC` value into it.

## `set_env`
Parameters:
- Any number of flags to append to the default set of flags (filtered)

`set_env` sets up the environment variables relevant to cross compilation. Most recipes will want to call this function before compiling anything.

The arguments are filtered first using the common filter syntax as with other functions; then using a custom syntax, based on what they start with:
- `c:value` appends `value` to `CFLAGS`
- `cxx:value` appends `value` to `CXXFLAGS`
- `p:value` appends `value` to `CPPFLAGS` (For the C preprocessor, not "C plus plus")
- `x:value` appends `value` to both `CFLAGS` and `CXXFLAGS`
- `link:value` appends `value` to `LDFLAGS`
- `r:value` appends `value` to `RUSTFLAGS`

As a special case, some common flags are recognized and put into appropriate env variables:
- `-l*`, `-L*`, `-flto`, `-fno-lto` and `-static` go into `CFLAGS`, `CXXFLAGS` and `LDFLAGS`
- `-I*` and `-D*` go into `CPPFLAGS`, `CFLAGS` and `CXXFLAGS`
- `-s`, `-g`, `-mtune=*`, `-marcH=*`, `-Wl,*`, `-Wno-*`, `-Ofast`, `-O[0123sz]` go into `CFLAGS` and `CXXFLAGS`

If none of the above are satisfied, an error is produced and the script exits.

Additionally, other important environment variables are set:
- `CC`, `CXX`, `LD`, `STRIP`, `AR`, `NM`, `RANLIB` and so on
- `CMAKE_MODULE_PATH`, `CMAKE_PREFIX_PATH`
- `PKG_CONFIG`, `PKG_CONFIG_PATH`
- `AB_CC`, `AB_CXX`, `AB_LD`, `$AB_NM` and so on (these values are used in wrapper scripts in the `wrappers` directory)

## `mkcd`
Parameters:
- Directory to create

`mkcd` creates a directory and enters it.
The directory and its parents will be created if missing.

## `msys_install`
Parameters:
- Has no parameters

`msys_install` installs the current directory into `$AB_PREFIX` under the assumption that it's an MSYS2 package downloaded internally using `download`.
Only use with recipes that specify MSYS2 binary sources.

## `append_pkgconfig`
Parameters:
- Any number of package names

`append_pkgconfig` runs `pkg-config` with appropriate flags to probe for flags specified in .pc files installed in `$AB_PREFIX`.
The output is appended to `CFLAGS` and `LDFLAGS`.

It's useful when a build script is unable to detect dependencies properly.

## `prepend_custom_prefix`
Parameters:
- A prefix path

Prepends a "prefix" by prepending `$CFLAGS`, `$CXXFLAGS` and `LDFLAGS` with flags that add search paths before others.
It's useful when you request a library be installed elsewhere (maybe to not conflict with the stuff in the global install prefix), and need to link with it.

See the recipe for `mpv` for a usage example.

## `ab-pkg-config`
Parameters:
- Any argument you want to pass to `pkg-config`

`ab-pkg-config` invokes `pkg-config`, after preparing the search paths.

## `apply_patches`
Parameters:
- The arguments to be forwarded to the `patch` command

`apply_patches` applies patches located in the `patches/<recipe>` directory (under the autobuild source tree).

## `need_excec`
Parameters:
- List of programs

Checks for existence of programs, exiting with an error if they're not found on the system.

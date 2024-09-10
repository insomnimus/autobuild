# Autobuild
Autobuild is a set of scripts and a frontend for cross compilation of statically-linked programs to Windows x64.

There are build recipes for dozens of applications and over 100 libraries.
Among the applications are:
- [FFmpeg](https://github.com/FFmpeg/FFmpeg) with nearly all optional libraries
- [Wget](https://www.gnu.org/software/wget/) and [Wget2](https://github.com/rockdaboot/wget2)
- [Curl](https://github.com/curl/curl) with 3 different TLS backends (Schannel, GnuTLS and LibreSSL), and HTTP 3 support with GnuTLS
- [Aria2](https://github.com/aria2/aria2)
- [MPV](https://github.com/mpv-player/mpv)
- [FluidSynth](https://github.com/FluidSynth/fluidsynth)
- [libarchive tools(bsdtar, bsdcat, bsdcpio)](https://github.com/libarchive/libarchive)
- [SoX](https://sourceforge.net/projects/sox/)

All applications are statically linked. There are no DLLs.

Note that this repository also contains apps that are trivial to build, as I have a personal automated build server and want to automatically build some of the stuff I use myself.

## Dependencies
Autobuild makes use of several commonly available programs:
- Bash version 5 or newer
- [Dash](https://wiki.archlinux.org/title/Dash) (Used in compiler wrapper scripts due to it being significantly faster than `sh`)
- GNU Make
- [Ninja](https://ninja-build.org)
- CMake version 3 or newer
- † [Meson](https://mesonbuild.com) version 1 or newer
- GNU Autotools: `autoconf`, `automake`, `libtool` etc
- [Autoconf archives for Autotools](https://github.com/autoconf-archive/autoconf-archive) (the macros are used by many configure scripts)
- † LLVM compiler suite version 16  or newer; must have `clang`, `clang++`, `llvm-windres`, `llvm-ar` and so on
- Rust version 1.65 or newer: `cargo` and `rustup`
- † The [MinGW-W64 cross compiler toolchain](https://www.mingw-w64.org) version 13 or newer
- the NASM assembler
- pkg-config
- Python3 (it's a very common dependency among build scripts, tho not used in Autobuild)
- GNU Sed
- The `patch` command from Difftools
- The `bsdtar` command from libarchive
- Wget

Some recipes might require additional programs that are not general enough to be listed here. Among these include `help2man`, `gperf` and `tclsh`.
It's also worth noting that some projects expect the `python` command to be available; if your distribution does not symlink `python` to `python3` these can fail to build.

† On Ubuntu (and likely Debian and other Debian based distributions), LLVM, Meson and MinGW packages are very old; you will have to either create a chroot of another distribution or manually install up to date versions of these tools (making sure to remove the system-provided package or shadow it in `$PATH`):
- You can obtain official up to date LLVM packages from [apt.llvm.org](https://apt.llvm.org).
	- However, packages obtained from LLVM's apt repositories do not create symlinks like `clang -> clang-17`; you have to do this yourself for every executable in `/usr/lib/llvm-17/bin`.
	- And you may have to uninstall the Ubuntu provided LLVM packages; some configure scripts still detect the older version of clang, even if you shadow it.
- Meson can be installed with `pip3 install --user meson`; however pip might refuse to do so if you have Meson installed through `apt`.
- For the MinGW toolchain, you can find pre-built portable archives from [github.com/xpack-dev-tools/mingw-w64-gcc-xpack](https://github.com/xpack-dev-tools/mingw-w64-gcc-xpack/) (it's unaffiliated and I do not specifically endorse it, it's just an option), or compile it from source.
- However FFMPEG currently does not build on Ubuntu for peculiar reasons.
- Advice: [Create an Arch Linux chroot instead](https://man.archlinux.org/man/mkarchroot.1.en). Autobuild works in Arch Linux out of the box.

## Usage
The only script you should run is `autobuild.sh`; do not try to run recipe scripts directly - `autobuild.sh` sets an environment up and invokes them for you.

Briefly, you provide a directory path for Autobuild to work in, and a recipe to build.
The directory will be used to store
- Source code of projects being built
- The installation "prefix" so that libraries can be reused across invocations
- The binaries that are built
- And other internal files such as helper programs that are compiled by autobuild for private use and a primitive filesystem based database

Since maintaining documentation on multiple places is tidious, the command line flags and options are not documented here; please run autobuild with the `--help` option to see all the flags and options.

Assuming a mainstream setup (no custom MinGW toolchain), to build `mpc` run the command
```shell
autobuild.sh --root ~/builds mpc
```

This will download the source code for `mpc` and all of its dependencies, cross compile them for Windows and archive it into a `tar.zst` (tar compressed using zstd) archive.

The archive will be written into `<root>/install/mpc/mpc.<version>.tar.zst`.

Some options you might want to mess with are
- `-M/--mingw-root`: Path to a custom MinGW toolchain sysroot; use it if you have MinGW-W64 installed in a non-standard location (you should probably not use it if you're not sure you need it).
- `--no-lto`: Disable link-time optimization; this may improve build times and memory use drastically at the cost of potentially missed optimizations. However the gains with LTO can be quite significant.
- `--cpu`: Optimize for a specific CPU; the value is passed to clang/gcc as `-march=CPU`, and rustc as `-Ctarget-cpu=CPU`. This has the potential to create drastically faster binaries, especially for codecs such as libmp3lame, libfdk-aac, libopus (which will cascade if building ffmpeg), but the produced executable might not run on other CPUs.
- `--tune`: Optimize for a specific CPU; unlike `--cpu` the produced binary should run on any x86_64 machine, but the gains aren't as profound.
- `--clean`: Clean the sources directory; removes build artifacts. Recommended every once in a while as by default the build directories are not cleaned.
- `-j/--jobs`: Number of parallel jobs; defaults to number of CPUs + 2. This value is used almost exclusively for `make` invocations. CMake and Meson based builds are not affected.

## Troubleshooting
Due to the nature of the project, there are tons of moving parts. Builds can break due to updates to source code, or due to unexpected circumstances relating to your setup.
I do not have the time or funding to respond to issues or feature requests but you can try debugging the problem yourself. Here are a few tips.
- The output of the script is written to `<root>/logs/<recipe>.log`; you can investigate it.
- The build directory is not cleaned automatically; you can usually find more logs in there. For example Autotools based projects will log into `config.log`.
- If you're having compilation errors (not linking), maybe due to errors in the source code, it might be the fault of the project. Some people do not anticipate or want to handle cross compilation to Windows.
- You can make Bash print everything it executes if you run the script with `bash -x autobuild.sh`.
- If you see that the build machine's libraries are attempted to be linked with, or system headers being imported, there's a good chance `pkg-config` could not find something on the autobuild prefix that's available system-wide (i.e. Linux libraries).
- If you're editing a library recipe, sometimes Autobuild's caching might get in your way. You can remove files in `<root>/data/libraries/<recipe>` to make Autobuild think it's not installed.
- It's also a good idea to remove `<root>/sources/<recipe>` if you change a recipe's download source in `repos.sh`.

## Internal Documentation
You can read the internal documentation [here](doc/index.md).
You really should read it if you intend to add new recipes or debug problems.

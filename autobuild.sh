#!/usr/bin/env bash

set -ue

# Variables
{
	AB_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
	if [[ ! $AB_DIR =~ ^[a-zA-Z0-9/+\-_@\.=]+$ ]]; then
		echo 1>&2 "error: The directory containing this script must not contain spaces or symbols special to shells"
		exit 1
	fi

	# Used by lib.sh to differentiate between apps and libraries
	APP=yes
	AB_LIB_FORCE=""
	AB_HOOK=""

	# Passed to configure as --build
	AB_BUILD_TRIPLE="$(gcc -dumpmachine 2>/dev/null || clang -dumpmachine 2>/dev/null || true)"
	AB_CPU=""
	AB_TUNE=sandybridge

	export AB_MINGW_SYSROOT=""
	AB_MINGW_CRT=""

	AB_WRAPPERS="$AB_DIR/wrappers"

	export AB_PREFIX=""
	AB_ROOT=""
	AB_TMP=""
	AB_DB=""
	AB_OPT=""
	AB_SOURCES=""
	AB_INSTALL=""
	AB_LOCAL=""

	AB_TAG=""
	AB_VERBOSE=""
	export AB_OPT_LEVEL=2
	AB_TOOLCHAIN=llvm
	AB_CMAKE=""
	AB_MESON=""
	AB_CARGO_CONFIG=""

	JOBS="$(nproc 2>/dev/null || echo 0)"
	((JOBS += 2))

	# Variables used in this script only.
	recipes=()
	force=""
	build_lib=""
	clean=0
	AB_RUST_PANIC=abort
	rust_codegen_units=1
	AB_RUST_BUILD_STD=0
}

# shellcheck source-path=SCRIPTDIR/lib.sh
. "$AB_DIR/lib.sh"

function show_help() {
	cat <<-END
		Autobuild: cross compile programs for Windows x86_64

		USAGE: autobuild --root=DIRECTORY [OPTIONS] <RECIPE>

		OPTIONS:
		  -R, --root=DIRECTORY: Directory used for downloaded source code, installed libraries and programs
		  -M, --mingw-root=DIRECTORY: The mingw-w64 cross compilers' sysroot
		   The directory must contain another directory named x86_64-w64-mingw32
		   This option is useful if you have a custom mingw sysroot
		   You don't need to use it if you obtained mingw-w64 with your package manager
		  -l, --lib: Build a library recipe
		  -O, --opt-level=0|1|2|3|s: Optimization level [default: $AB_OPT_LEVEL]
		  --lto: Enable link time optimizations; this can yield faster executables but the linking times and memory use  will grow. Note that LTO isn't always correct: due to the bugs in the compilers, it's more likely to break and produce executables that don't work right
		  --tune=CPU: Tune for a specific CPU (accepts the same names as gcc -mtune=) [default: $AB_TUNE if --cpu is not set]
		  --cpu=CPU: Hard-optimize for the given CPU (accepts the same names as gcc -march=); this would produce executables that might not run on other CPUs, but can be faster. Note that using this with --lto especially with ffmpeg (or its dependencies) is observed to create faulty executables
		  -j, --jobs=N: Maximum number of jobs while invoking make [default: $JOBS]
		  --clean: Clean build artifacts (non destructive)
		  -f, --force: Force a build even if the latest version of the recipe is installed
		  -v, --verbose: Be more verbose
		  --rust-panic=abort|unwind: Whether rust programs should abort or unwind on panics [default: $AB_RUST_PANIC]
		  --rust-codegen-units=N: Number of Rust code generation units; lower values are likely to produce faster or smaller binaries at the expense of compilation time [default: $rust_codegen_units]
		  --rust-build-std: Build the Rust standard library from source for Rust programs (may improve performance and reduce binary size at the expense of compilation time)
		    This will install the unstable toolchain and the rust-src component with rustup if it's missing
		    This is experimental and might fail to build
		  -h, --help: Show this message and exit
		  -H, --hook=COMMAND: Run COMMAND if an app recipe completes and creates an archive
		   The following environment variables will be set for the command:
		   \$RECIPE: The name of the recipe as provided to this script
		   \$VERSION: The version string or git tag of the app that was built
			\$ARCHIVE_PATH: Full path to the generated release archive
		   COMMAND will be executed as if typed on a Bash prompt
	END
}

function err_exit() {
	echo 1>&2 "error: $*"
	exit 1
}

# Parse args.
{
	ARGV=()
	while [[ $# != 0 ]]; do
		case "$1" in
		"") ;; # Ignore empty args
		--*=*)
			ARGV+=("${1%%=*}" "${1#*=}")
			;;
		-[!-]*)
			for ((i = 1; i < ${#1}; i++)); do
				c="${1:i:1}"
				ARGV+=("-$c")
				if [[ $c == [RMHjO] && $i -lt ${#1}-1 ]]; then
					# It's a known option with a value attached.
					ARGV+=("${1:i+1}")
					break
				fi
			done
			;;
		*)
			ARGV+=("$1")
			;;
		esac
		shift
	done

	set -- "${ARGV[@]}"
	unset ARGV

	lto=0
	is_tune_set=0
	while [[ $# -gt 0 ]]; do
		case "$1" in
		-h | --help)
			show_help
			exit
			;;
		-l | --lib) build_lib=yes ;;
		-f | --force) force=yes ;;
		-v | --verbose)
			AB_VERBOSE=1
			export AB_HELPER_VERBOSE="${AB_HELPER_VERBOSE:-1}"
			;;
		--clean) clean=1 ;;
		--lto) lto=1 ;;
		--cpu)
			if ! shift; then
				err_exit "--cpu requires a value but none was provided"
			fi
			AB_CPU="$1"
			if [[ ! $AB_CPU =~ ^[a-zA-Z0-9][a-zA-Z0-9+_-]*$ ]]; then
				err_exit "--cpu: value doesn't look like a valid cpu name: $AB_CPU"
			fi
			;;
		--tune)
			if ! shift; then
				err_exit "--cpu requires a value but none was provided"
			fi
			AB_TUNE="$1"
			if [[ ! $AB_TUNE =~ ^[a-zA-Z0-9][a-zA-Z0-9+_-]*$ ]]; then
				err_exit "--tune: value doesn't look like a valid cpu name: $AB_TUNE"
			fi
			is_tune_set=1
			;;
		-O | --opt-level)
			if ! shift; then
				err_exit "-O --rust-panic requires a value but none was provided"
			fi
			if [[ $1 != [0123s] ]]; then
				err_exit "the optimization level must be one of 0, 1, 2, 3 or s; got $1"
			fi
			AB_OPT_LEVEL="$1"
			;;
		--rust-build-std) AB_RUST_BUILD_STD=1 ;;
		--rust-codegen-units)
			if ! shift; then
				err_exit "--rust-codegen-units requires a value but none was provided"
			fi
			if [[ ! $1 =~ ^[1-9][0-9]*$ ]]; then
				err_exit "--rust-codegen-units must be a positive integer"
			fi
			rust_codegen_units="$1"
			;;
		--rust-panic)
			if ! shift; then
				err_exit "--rust-panic requires a value but none was provided"
			fi
			if [[ $1 != abort && $1 != unwind ]]; then
				err_exit "--rust-panic must be one of abort or unwind; got $1"
			fi
			AB_RUST_PANIC="$1"
			;;
		-M | --mingw-root)
			if ! shift; then
				err_exit "-M --mingw-root requires a value but none was provided"
			fi
			AB_MINGW_SYSROOT="$(realpath -- "$1")" || exit
			;;
		-R | --root)
			if ! shift; then
				err_exit "-R --root requires a value but none was provided"
			fi
			AB_ROOT="$1"
			;;
		-H | --hook)
			if ! shift; then
				err_exit "the option -H --hook requires a value"
			fi
			AB_HOOK="$1"
			;;
		-j | --jobs)
			if ! shift; then
				err_exit "the option -j --jobs requires a value"
			fi
			JOBS="$1"
			if [[ ! $JOBS =~ ^[0-9]+$ ]]; then
				err_exit "value passed to -j --jobs must be a non-negative integer"
			fi
			;;
		-*)
			err_exit "unrecognized option $1"
			;;
		*)
			recipes+=("$1")
			;;
		esac
		shift
	done
}

# Validate arguments.
{
	# Don't let root continue any further.
	if [[ ${AUTOBUILD_ALLOW_ROOT:-} != 1 && ${USER:-"$(whoami || true)"} == root ]]; then
		eprint "error: running this script as root is dangerous"
		eprint "If you wish to proceed anyway, set the AUTOBUILD_ALLOW_ROOT environment variable to 1"
		exit 1
	fi

	if [[ $lto == 1 ]]; then
		BASE_FLAGS+=(x:-flto)
	fi

	if [[ -n $AB_CPU ]]; then
		BASE_FLAGS+=(x:-march="$AB_CPU" r:-Ctarget-cpu="$AB_CPU")
		if [[ $is_tune_set == 1 ]]; then
			BASE_FLAGS+=(x:-mtune="$AB_TUNE")
		fi
	else
		BASE_FLAGS+=(x:-mtune="$AB_TUNE")
	fi

	BASE_FLAGS+=("-O$AB_OPT_LEVEL")

	if [[ -z $AB_ROOT ]]; then
		err_exit "--root is required"
	fi

	AB_ROOT="$(realpath -- "$AB_ROOT")"
	AB_INSTALL="$AB_ROOT/install"
	AB_PREFIX="$AB_ROOT/libraries"
	AB_OPT="$AB_ROOT/opt"
	AB_DB="$AB_ROOT/data"
	AB_SOURCES="$AB_ROOT/sources"
	AB_LOCAL="$AB_ROOT/local"

	# Check for incorrect options.
	if [[ ! $AB_ROOT =~ ^[a-zA-Z0-9/+\-_@\.=]+$ ]]; then
		err_exit "--root must not contain special characters or whitespace"
	elif [[ -n $AB_MINGW_SYSROOT && ! $AB_MINGW_SYSROOT =~ ^[a-zA-Z0-9/+\-_@\.=]+$ ]]; then
		err_exit "--mingw-root must not contain special characters or whitespace"
	elif [[ -n $AB_MINGW_SYSROOT && ! -d $AB_MINGW_SYSROOT ]]; then
		err_exit "--mingw-root does not point to a directory"
	elif [[ -n $AB_MINGW_SYSROOT && ! -e "$AB_MINGW_SYSROOT/x86_64-w64-mingw32" ]]; then
		err_exit "--mingw-root: directory does not contain x86_64-w64-mingw32"
	fi

	# Deduplicate arguments.
	declare -A map
	_recipes=()
	for r in "${recipes[@]}"; do
		if [[ ${map["$r"]:-} ]]; then
			continue
		fi
		map["$r"]=1
		_recipes+=("$r")
		if [[ $clean == 1 && ! -e $AB_SOURCES/$r ]]; then
			err_exit "no $r in $AB_SOURCES"
		elif [[ $clean == 1 ]]; then
			continue
		fi

		if [[ $build_lib && ! -f "$AB_DIR/lib-recipes/$r" && ! -f "$AB_DIR/multi-recipes/$r" ]]; then
			if [[ -f "$AB_DIR/app-recipes/$r" ]]; then
				err_exit "$r is not a library; if you meant to build an app, do not specify --lib"
			else
				err_exit "$r: no such recipe"
			fi
		elif [[ ! $build_lib && ! -f "$AB_DIR/app-recipes/$r" && ! -f "$AB_DIR/multi-recipes/$r" ]]; then
			if [[ -f "$AB_DIR/lib-recipes/$r" ]]; then
				err_exit "$r is not an application; if you meant to build a library, specify --lib"
			else
				err_exit "$r: no such recipe"
			fi
		fi
	done

	recipes=("${_recipes[@]}")
}

if [[ $clean == 1 ]]; then
	need_exec git
	if [[ ${#recipes[@]} != 0 ]]; then
		(
			for r in "${!recipes[@]}"; do
				if [[ -d "$AB_SOURCES/$r" ]]; then
					cd "$AB_SOURCES/$r"
					reset_repo &>/dev/null || true
				fi
			done
		)
	else
		(
			shopt -s nullglob
			for p in "$AB_SOURCES"/*; do
				if [[ -d $p ]]; then
					cd "$p"
					reset_repo &>/dev/null || true
				fi
			done
		)
	fi
	exit
fi

if [[ ${#recipes[@]} == 0 ]]; then
	err_exit "no recipe was specified"
elif [[ $build_lib != yes && "${#recipes[@]}" != 1 ]]; then
	err_exit "only one app can be built at once"
fi

# Check for required programs and versions
{
	need_exec clang clang++ ld.lld llvm-{ar,strip,nm,ranlib,windres} \
		nasm x86_64-w64-mingw32-gcc \
		autoreconf autopoint cmake make ninja meson pkg-config \
		rustup cargo install \
		git wget bsdtar python3 \
		sed patch dash

	for script in "$AB_DIR/requirement-checks"/*; do
		if ! "$script" 1>&2; then
			exit 1
		fi
	done
}

# Check default linker and include paths.
{
	s="$(x86_64-w64-mingw32-gcc -print-search-dirs | grep ^libraries:)"
	s="${s#libraries: =}"
	split paths "$s" ":"
	for p in "${paths[@]}"; do
		if [[ -e $p ]]; then
			p="$(realpath -- "$p")"
			verbose "adding to library path: $p"
			AB_GCC_LIBS+=("-L$p")
		fi
	done
}

mkdir -p -- "$AB_INSTALL" "$AB_ROOT/logs" "$AB_DB" "$AB_SOURCES" "$AB_PREFIX" "$AB_OPT" "$AB_LOCAL"/bin
# Check if ab-helper is up to date.
(
	build_ab_helper=1
	latest_ab_helper_version="$(sed -rn 's/^ *version *= *"(.+)"$/\1/p' "$AB_DIR/ab-helper/Cargo.toml")"
	if type "$AB_LOCAL/bin/ab-helper" &>/dev/null; then
		verbose "checking if ab-helper is up to date"
		output="$("$AB_LOCAL/bin/ab-helper" version || echo _no_version_)"
		if [[ $output == "$latest_ab_helper_version" ]]; then
			verbose "ab-helper is up to date: $output"
			build_ab_helper=0
		else
			info "detected update to ab-helper: $output -> $latest_ab_helper_version"
		fi
	else
		info "ab-helper not found in $AB_LOCAL; installing"
	fi

	if [[ $build_ab_helper == 1 ]]; then
		info "building ab-helper for local use"
		run cargo install --root="$AB_LOCAL" --force --path "$AB_DIR/ab-helper" || {
			err_exit "failed to build required tool ab-helper; please verify that your rust toolchain is up to date and functional"
		}

		# Clean the build directory for the user's sake.
		cargo clean --manifest-path "$AB_DIR/ab-helper/Cargo.toml" &>/dev/null || true
	fi

	# Ensure wrapper symbolic links.
	for a in ab-{clang,clang++,gcc,g++,lld,ld,pkg-config} x86_64-w64-mingw32-pkg-config; do
		ln -sf ab-helper "$AB_LOCAL/bin/$a" || {
			eprint "error: failed to create wrapper symbolic link $a in $AB_LOCAL/bin"
			exit 1
		}
	done

)

export PATH="$AB_WRAPPERS:$AB_LOCAL/bin:${AB_MINGW_SYSROOT:+$AB_MINGW_SYSROOT/x86_64-w64-mingw32}:$PATH"
unset CARGO_TARGET_DIR
AB_TMP="$(mktemp -d autobuild_XXXXXX --tmpdir)"
mkdir -- "$AB_TMP/checked-for-updates"
AB_CARGO_CONFIG="$AB_TMP/cargo-config.toml"
ab-helper toml set -p "$AB_DIR/config/cargo-config.toml" -o "$AB_CARGO_CONFIG" \
	"profile.release.panic = '$AB_RUST_PANIC'" \
	"profile.release.codegen-units = $rust_codegen_units" \
	"profile.release.opt-level = $(
		if [[ $AB_OPT_LEVEL == [0123] ]]; then
			echo "$AB_OPT_LEVEL"
		else
			echo "'$AB_OPT_LEVEL'"
		fi
	)"

{
	verbose "probing for the default C runtime"
	exe="$AB_TMP/crt_test.exe"
	clang --target=x86_64-w64-mingw32 \
		"${AB_MINGW_SYSROOT:+--sysroot=$AB_MINGW_SYSROOT}" \
		"${AB_GCC_LIBS[@]}" \
		-fuse-ld=lld -o "$exe" -x c - <<-'EOF'
			int main() { return 0; }
		EOF

	AB_MINGW_CRT="$(ab-helper detect-crt -- "$exe")" || {
		rm -rf -- "$AB_TMP"
		exit 1
	}
	verbose "detected default C runtime: $AB_MINGW_CRT"
}

if [[ $build_lib ]]; then
	for r in "${recipes[@]}"; do
		if [[ ! -f "$AB_DIR/lib-recipes/$r" && ! -f "$AB_DIR/multi-recipes/$r" ]]; then
			err_exit "$r: no such library recipe"
		fi
	done

	APP=""
	AB_LIB_FORCE="" install_lib dlfcn winpthreads

	for r in "${recipes[@]}"; do
		logs="$AB_ROOT/logs/$r.log"
		{
			echo "$r (library)"
			date +"build started at %Y-%m-%d %H:%M"
		} >"$logs"
		AB_LIB_FORCE="$force" install_lib "$r" 2>&1 | tee -a "$logs"
		if [[ ${PIPESTATUS[0]} != 0 ]]; then
			echo "recipe failed" >>"$logs"
			rm -rf "$AB_TMP"
			exit 1
		else
			echo "recipe completed" >>"$logs"
			echo "$r: success"
		fi
	done
	rm -rf "$AB_TMP"
	exit 0
fi

export RECIPE="${recipes[0]}"

function execute_recipe() {
	# cd -- "$AB_SOURCES/$RECIPE" || return
	(
		set -ue
		install_lib dlfcn winpthreads
		PATCHES="$AB_DIR/patches/$RECIPE"
		LIB_ID=""
		if [[ -f "$AB_DIR/app-recipes/$RECIPE" ]]; then
			. "$AB_DIR/app-recipes/$RECIPE"
		else
			. "$AB_DIR/multi-recipes/$RECIPE"
			if [[ -n $LIB_ID ]]; then
				db.set "$RECIPE" "$LIB_ID"
			fi
		fi
	)
}

set -ue

download "$RECIPE"
# cd -- "$AB_SOURCES/$RECIPE"
AB_SAVETO="$AB_INSTALL/$RECIPE/$RECIPE.$AB_TAG.tar.zst"
if [[ -e $AB_SAVETO && $force != yes ]]; then
	eprint "the output file $AB_SAVETO already exists"
	eprint "specify --force to overwrite"
	exit 0
fi

logs="$AB_ROOT/logs/$RECIPE.log"
{
	printf '%s %s\n' "$RECIPE" "$AB_TAG"
	date +"build started at %Y-%m-%d %H:%M"
} >"$logs"

unset_env
execute_recipe 2>&1 | tee -a -- "$logs"

if [[ ${PIPESTATUS[0]} != 0 ]]; then
	tee -a -- "$logs" <<<"recipe failed"
	rm -rf "$AB_TMP"
	exit 1
else
	tee -a -- "$logs" <<<"recipe completed"
	rm -rf "$AB_TMP"
fi

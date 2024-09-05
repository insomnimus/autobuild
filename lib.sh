#!/usr/bin/bash

# shellcheck disable=SC2031,SC2155
# shellcheck source-path=SCRIPTDIR
. "${AB_DIR:?}/repos.sh"

AB_HAVE_RUSTUP_WINDOWS_TARGET=0
AB_GCC_LIBS=()
BASE_FLAGS=(
	gcc:x:-fmax-errors=5
	llvm:x:-ferror-limit=5
	gcc:-fno-lto llvm:x:-flto
	# llvm:x:-Wl,--error-limit=10
	-s p:-DNDEBUG
	x:{-g0,-w}
)

function info() {
	echo 1>&2 "> $*"
}

function eprint() {
	echo 1>&2 "$*"
}

function error() {
	eprint "autobuild error: $*"
	exit 5
}

function run() {
	eprint "+ $*"
	"$@"
}

function run_verbose() {
	if [[ $AB_VERBOSE == 1 ]]; then
		eprint "+ $*"
	fi
	"$@"
}

function verbose() {
	if [[ $AB_VERBOSE == 1 ]]; then
		echo 1>&2 "! $*"
	fi
}

function remove_prefix() {
	set -u
	local s="$1"
	local prefix="$2"
	if [[ $s == "$prefix"* ]]; then
		printf %s "${s:${#prefix}}"
	else
		printf %s "$s"
	fi
}

function split() {
	set -u
	if [[ $# -gt 3 ]]; then
		error "too many arguments passed to split"
	fi
	local __out="$1"
	local __s="$2"
	local __sep="$3"
	if [[ ${#__sep} != 1 ]]; then
		error "split: delimiter must be a single character"
	elif [[ $__sep == $'\n' ]]; then
		error "separator passed to split cannot be \\n"
	fi

	IFS="$__sep" read -r -a "${__out:?}" <<<"$__s"
}

function array.contains() {
	local x="$1"
	shift || return 1
	local item
	for item in "$@"; do
		if [[ $x == "$item" ]]; then
			return 0
		fi
	done
	return 1
}

function join_by() {
	local d=${1-}
	local f=${2-}
	if shift 2; then
		printf %s "$f" "${@/#/$d}"
	fi
}

function set_env() {
	set -u
	AB_CMAKE="$AB_DIR/config/$AB_TOOLCHAIN.cmake"
	AB_MESON="$AB_TMP/$AB_TOOLCHAIN.meson"
	if [[ ! -f $AB_MESON ]]; then
		sed -r "s/^ *cpu *= *'sandybridge' *\$/cpu = '$AB_CPU'/" "$AB_DIR/config/$AB_TOOLCHAIN.meson" >"$AB_MESON"
	fi

	local m=x86_64-w64-mingw32
	local p="${AB_MINGW_SYSROOT:-/usr}/bin"
	prepend_path "$AB_WRAPPERS" "$p"

	case "$AB_TOOLCHAIN" in
	gcc)
		export CC=$m-gcc \
			CXX=$m-g++ \
			AR=$m-gcc-ar \
			WINDRES=$m-windres RC=$m-windres \
			STRIP=$m-strip \
			RANLIB=$m-gcc-ranlib \
			DLLTOOL=$m-dlltool \
			DLLWRAP=$m-dllwrap \
			LD=$m-ld \
			NM=$m-gcc-nm
		;;

	llvm)
		# export CC="clang $x" \
		# CXX="clang++ $x" \
		export CC=ab-clang CXX=ab-clang++ \
			LD=ld.lld \
			STRIP=ab-llvm-strip \
			AR=llvm-ar \
			NM=llvm-nm \
			RANLIB=llvm-ranlib \
			DLLTOOL=llvm-dlltool \
			WINDRES="llvm-windres --target=pe-x86-64" RC="llvm-windres --target=pe-x86-64" \
			RC="llvm-windres --target=pe-x86-64" RC="llvm-windres --target=pe-x86-64" \
			OBJDUMP=llvm-objdummp \
			MT=llvm-mt
		;;
	*)
		error "unknown toolchain: $AB_TOOLCHAIN"
		;;
	esac

	export CMAKE_LIBRARY_PATH="$AB_PREFIX/lib" \
		CMAKE_MODULE_PATH="$AB_PREFIX/lib/cmake" \
		CMAKE_PREFIX_PATH="$AB_PREFIX" \
		PKG_CONFIG_PATH="$AB_PREFIX/lib/pkgconfig:$AB_PREFIX/share/pkgconfig" \
		PKG_CONFIG=x86_64-w64-mingw32-pkg-config

	export AB_CC="$CC" AB_CXX="$CXX" AB_LD="$LD" AB_STRIP="$STRIP" \
		AB_AR="$AR" AB_RANLIB="$RANLIB" AB_NM="$NM" AB_WINDRES="$WINDRES"

	set_flags "$@" "${AB_GCC_LIBS[@]}"
	# gcc:x:-B"$AB_WRAPPERS/gcc-prefix/"
}

function unset_env() {
	unset CC CXX CPP {AB_,}{C,CXX,CPP,LD,AR}FLAGS \
		AB_{CC,CXX,CPP,LD,AR,RANLIB,DLLTOOL,NM,STRIP} AB_LDFLAGS_AFTER AB_LDFLAGS_BEFORE \
		DLLTOOL DLLWRAP NM AR LD RANLIB STRIP \
		CARGO_TARGET_DIR CARGO_OUT_DIR RUSTFLAGS RUST_LOG RUST_BACKTRACE \
		PKG_CONFIG PKG_CONFIG_PATH CMAKE_LIBRARY_PATH CMAKE_MODULE_PATH CMAKE_PREFIX_PATH
}

function prepend_path() {
	set -u
	local p
	local list=("$@")
	local -A in_list
	for p in "$@"; do
		in_list["$p"]=1
	done
	local cur_path
	split cur_path "$PATH" ":"
	for p in "${cur_path[@]}"; do
		if [[ -z $p ]]; then
			continue
		fi
		if [[ ${!in_list["$p"]:-0} == 0 ]]; then
			list+=("$p")
			in_list["$p"]=1
		fi
	done

	PATH="$(join_by ":" "${list[@]}")"
}

function prepend_custom_prefix() {
	local p="${1:?}"
	if [[ $# -gt 1 ]]; then
		error "too many arguments passed to prepend_custom_prefix"
	fi

	export \
		CFLAGS="-L$p/lib -I$p/include ${CFLAGS:-}" \
		AB_CFLAGS="-L$p/lib -I$p/include ${AB_CFLAGS:-}" \
		CXXFLAGS="-L$p/lib -I$p/include ${CXXFLAGS:-}" \
		AB_CXXFLAGS="-L$p/lib -I$p/include ${AB_CXXFLAGS:-}" \
		CPPFLAGS="-I$p/include ${CPPFLAGS:-}"
	LDFLAGS="-L$p/lib ${LDFLAGS:-}" \
		AB_LDFLAGS="-L$p/lib ${AB_LDFLAGS:-}" \
		PKG_CONFIG_PATH="$p/lib/pkgconfig:$p/share/pkgconfig:${PKG_CONFIG_PATH:-}" \
		CMAKE_MODULE_PATH="$p/lib/cmake:${CMAKE_MODULE_PATH:-}" \
		CMAKE_LIBRARY_PATH="$p/lib:${CMAKE_LIBRARY_PATH:-}"
}

function reset_repo() {
	git restore . &&
		git clean -qffxd &&
		git reset --hard --quiet &&
		git submodule -q foreach --recursive git clean -qffxd &&
		git submodule -q foreach --recursive git reset --hard --quiet
}

function latest_git_tag() {
	local sort="${1:-}"
	local sort_key

	case "$sort" in
	"")
		git describe --tags --abbrev=0
		return
		;;
	sort=date) sort_key=taggerdate ;;
	sort=version) sort_key=v:refname ;;
	sort=*) error "unknown way to sort git tags: $sort; allowed values are 'date' and 'version'" ;;
	*) error "unrecognized option $sort" ;;
	esac

	local output
	output="$(git tag --list --sort=-"$sort_key")" || return
	if [[ -z $output ]]; then
		eprint "error: no git tags found"
		return 1
	fi
	printf '%s\n' "${output%%$'\n'*}"
}

function _clone_master() {
	set -ue

	# Clear AB_LIB_FORCE so only the top-level library is affected.
	local _force="$AB_LIB_FORCE"
	AB_LIB_FORCE=""

	local name="$1"
	local git="$2"
	local test="${3:-}"
	local branch="${4:-}"
	if [[ $APP && ! -f "$AB_DIR/version-scripts/$name" ]]; then
		error "cannot build app recipes from git master unless there's a version script"
	elif [[ ! $APP && -z $test ]]; then
		error "missing required argument: path to test for"
	fi

	local current_commit latest_commit installed_id
	installed_id="$(db.get "$name" || true)"

	cd -- "$AB_SOURCES"
	if [[ -e $name ]]; then
		cd -- "$name"
		reset_repo
		current_commit="$(git rev-parse --short HEAD)"

		git pull -q
		git submodule update --recursive --quiet
		latest_commit="$(git rev-parse --short HEAD)"
		LIB_ID="$latest_commit"
		check_version_script "$name"

		if [[ 
			-z $APP &&
			! $_force &&
			$installed_id == "$LIB_ID" &&
			$current_commit == "$latest_commit" &&
			-e "$AB_PREFIX"/$test ]]; then
			info "$name is up to date"
			exit 0
		fi
		add_checked "$name"
	else
		info "cloning $name from $git"
		git clone --recursive --quiet "$git" "$name"
		cd "$name"
		if [[ -n "$branch" ]]; then
			git checkout -q "$branch"
		fi
		add_checked "$name"
		LIB_ID="$(git rev-parse --short HEAD)"
		check_version_script "$name"
	fi
}

function install_lib() {
	local l name
	for l in "$@"; do
		if ! name="$(filter "$l")"; then
			info "skipping install: $l"
			continue
		fi

		set -ue
		eprint "+ install_lib $name"
		local script="$AB_DIR/lib-recipes/$name"
		if [[ ! -e $script ]]; then
			script="$AB_DIR/multi-recipes/$name"
			if [[ ! -e $script ]]; then
				error "no recipe script found for $name"
			fi
		fi

		# IMPORTANT: Do not use if ! () and do not set -e here - it inhibits `set -e` inside the subshell!
		# Also do not do || code=$? it has the same unfortunate effect.
		set +e
		local code=0
		# shellcheck disable=SC2030,SC2034,SC1090
		(
			set -ue
			unset_env
			cd "$AB_SOURCES"
			APP=""
			PATCHES="$AB_DIR/patches/$name"
			LIB_ID=""
			. "$script"
			if [[ -n $LIB_ID ]]; then
				# We successfully installed a new version.
				db.set "$name" "$LIB_ID"
			fi
		)
		code=$?
		set -e
		if [[ $code != 0 ]]; then
			eprint "library recipe $name failed ($script)"
			return 1
		fi
	done
}

function out() {
	if [[ -z $APP ]]; then
		return
	fi
	(
		set -ue
		local tmp
		tmp="$(mktemp -d autobuild_outXXXXXX --tmpdir)"
		local f
		for f in "$@"; do
			if [[ $f == *.exe || $f == *.dll ]]; then
				${STRIP:-llvm-strip} "$f" &>/dev/null || true
			fi
		done

		cp -rf -- "$@" "$tmp/"
		pushd >/dev/null "$tmp"
		if [[ -e $AB_SAVETO ]]; then
			verbose "deleting old archive"
			rm -f -- "$AB_SAVETO"
		else
			mkdir -p "$(dirname -- "$AB_SAVETO")"
		fi
		local code=0
		info "creating archive"
		# run_verbose 7z -bso0 -bsp0 a "${AB_SAVETO:?}" -- * || code=$?
		run_verbose bsdtar -caf "$AB_SAVETO" --options zstd:compression-level=15,zstd:threads="$JOBS" ./* || code=$?
		popd >/dev/null
		rm -rf "$tmp"
		if [[ $code != 0 ]]; then
			return "$code"
		fi

		if [[ -n $AB_HOOK ]]; then
			info "running hook"
			RECIPE="${RECIPE:?}" VERSION="${AB_TAG:?}" ARCHIVE_PATH="$AB_SAVETO" run $AB_HOOK || {
				eprint "hook exited with $?"
				info "removing installed archive"
				run rm -f -- "$AB_SAVETO"
				return 1
			}
		fi
		info "all done"
	)
}

function prepare_autotools() {
	set -ue
	local f
	for f in bootstrap.sh autogen.sh bootstrap autogen; do
		if [[ -e $f ]]; then
			info "found script $f"
			run "./$f"
			return
		fi
	done

	info "no bootstrap or autogen, attempting autoreconf"
	run autoreconf -fi
}

function run_cmake() {
	set -ue
	local args=(
		-G Ninja
		-DCMAKE_BUILD_TYPE=Release
		-DCMAKE_INSTALL_PREFIX="${AB_PREFIX:?}"
		-DCMAKE_TOOLCHAIN_FILE="${AB_CMAKE:?}"
		-DCMAKE_FIND_ROOT_PATH="$AB_PREFIX${AB_MINGW_SYSROOT:+:$AB_MINGW_SYSROOT/x86_64-w64-mingw32}"
		-DCMAKE_FIND_ROOT_PATH_MODE_PROGRAM=NEVER
		-DPKG_CONFIG_ARGN=--static
	)

	for a in "$@"; do
		if ! a="$(filter "$a")"; then
			continue
		fi

		args+=("$a")
	done
	unset a

	eprint "+ cmake ${args[*]}"
	cmake "${args[@]}"
}

function cmake_install() {
	set -ue
	local path="$1"
	shift
	run_cmake "$path" "$@"
	run cmake --build . --config Release
	run cmake --install .
}

function run_configure() {
	set -ue
	local path="$1"
	if [[ $path == configure || $path == autogen.sh ]]; then
		path="./$path"
	elif [[ ! $path =~ /(configure|autogen\.sh) ]]; then
		eprint "autobuild error: the first argument to run_configure must be the configure script path"
		return 1
	fi

	shift
	if [[ ! -x $path ]]; then
		info "$path not found, attempting autogen/bootstrap/autoreconf"
		prepare_autotools
	fi

	args=(
		--host=x86_64-w64-mingw32 --target=x86_64-w64-mingw32
		--enable-static --disable-shared
		--prefix="$AB_PREFIX"
		--disable-dependency-tracking
	)
	if [[ -n $AB_BUILD_TRIPLE ]]; then
		args+=("--build=$AB_BUILD_TRIPLE")
	fi

	local a
	for a in "$@"; do
		if ! a="$(filter "$a")"; then
			continue
		fi

		if [[ $a != --with-gnu-ld || $AB_TOOLCHAIN == gcc ]]; then
			args+=("$a")
		fi
	done

	# Some configure scripts employ bashisms that don't work with dash.
	if [[ ${USE_DASH:-1} != 0 ]]; then
		AB_CFLAGS+=" -O0 -fno-lto" AB_CXXFLAGS+="-O0 -fno-lto" AB_LDFLAGS+=" -fno-lto" run dash -f "$path" "${args[@]}"
	else
		AB_CFLAGS+=" -O0 -fno-lto" AB_CXXFLAGS+="-O0 -fno-lto" AB_LDFLAGS+=" -fno-lto" run "$path" "${args[@]}"
	fi
}

function configure_build_par() {
	set -ue
	run_configure ./configure "$@"
	run make -j"$JOBS"
}

function configure_install() {
	set -ue
	run_configure ./configure "$@"
	run make -j"${configure_install_jobs:-1}"
	run make install
}

function configure_install_par() {
	configure_install_jobs="$JOBS" configure_install "$@"
}

function mkcd() {
	local dir="${1:?autobuild error: missing required argument 1: directory}"
	if [[ -n ${2:-} ]]; then
		eprint "autobuild error: more than one argument passed to mkcd"
		return 1
	fi

	printf 1>&2 '+ mkcd %q\n' "$dir"
	mkdir -p -- "$dir"
	cd -- "$dir"
}

function meson_build() {
	set -ue
	local args=(
		--cross-file "$AB_MESON"
		--prefix "$AB_PREFIX"
		--buildtype plain
		--default-library static
		--prefer-static
	)

	local a
	for a in "$@"; do
		if ! a="$(filter "$a")"; then
			continue
		fi

		args+=("$a")
	done

	run meson setup . autobuild "${args[@]}"
	run meson compile -C autobuild
}

function meson_install() {
	set -ue
	meson_build "$@"
	run meson install -C autobuild
}

function cargo_build() {
	(
		set -ue
		ensure_rust_target
		local rf=("-L:native=$AB_PREFIX/lib")

		local a
		for a in "${AB_GCC_LIBS[@]}" "${BASE_FLAGS[@]}"; do
			case "$a" in
			r:*) rf+=("${a:2}") ;;
			l:*) rf+=("-Clink-arg=${a:2}") ;;
			-L?*) rf+=("-L:native=${a:2}") ;;
			-l:*.a) rf+=("-l:static=${a:2:${#a}-5}") ;;
			-l*) rf+=("-l:static=${a:2}") ;;
			esac
		done

		local release=--release
		for a in "$@"; do
			if [[ $a == --profile || $a == --profile=* ]]; then
				release=""
				break
			fi
		done

		local toolchain=nightly
		local build_std="-Zbuild-std=std,core,alloc"
		if [[ $AB_RUST_BUILD_STD == 0 ]]; then
			toolchain=stable
			build_std=""
		elif [[ $AB_RUST_PANIC == abort ]]; then
			build_std+=",panic_abort"
		fi

		RUSTFLAGS="${rf[*]}" \
			AR=llvm-ar \
			PKG_CONFIG_ALLOW_CROSS_x86_64_pc_windows_gnu=1 \
			PKG_CONFIG_x86_64_pc_windows_gnu=ab-pkg-config \
			TARGET_CC=ab-clang \
			run cargo "+$toolchain" rustc \
			$release \
			$build_std \
			--target x86_64-pc-windows-gnu \
			--config "$AB_CARGO_CONFIG" \
			"$@"
	)
}

function cargo_cinstall() {
	ensure_rust_target || return

	# Ensure cargo-c is available and up to date
	(
		set -ue

		local need_build=1
		verbose "checking if cargo-c is installed and up to date"
		if ! type "$AB_LOCAL/bin/cargo-cinstall" &>/dev/null; then
			info "could not find cargo-c"
		else
			local installed_version upstream_version
			installed_version="$(cargo-cinstall --version | awk '{print $2}')"
			upstream_version="$(ab-helper crate-version cargo-c)"

			if [[ $installed_version == "$upstream_version" ]]; then
				need_build=0
			else
				verbose "detected update to cargo-c: $installed_version -> $upstream_version"
			fi
		fi

		if [[ $need_build == 0 ]]; then
			verbose "cargo-c is up to date"
		else
			info "installing cargo-c"
			unset_env
			cargo install cargo-c --root "$AB_LOCAL" --force
		fi
	)

	(
		set -ue
		local rf=("-L:native=$AB_PREFIX/lib")

		local a
		for a in "${AB_GCC_LIBS[@]}" "${BASE_FLAGS[@]}"; do
			case "$a" in
			r:*) rf+=("${a:2}") ;;
			l:*) rf+=("-Clink-arg=${a:2}") ;;
			-L?*) rf+=("-L:native=${a:2}") ;;
			-l:*.a) rf+=("-l:static=${a:2:${#a}-5}") ;;
			-l*) rf+=("-l:static=${a:2}") ;;
			esac
		done

		local release=--release
		for a in "$@"; do
			if [[ $a == --profile || $a == --profile=* ]]; then
				release=""
				break
			fi
		done

		local toolchain=nightly
		local build_std="-Zbuild-std=std,core,alloc"
		if [[ $AB_RUST_BUILD_STD == 0 ]]; then
			toolchain=stable
			build_std=""
		elif [[ $AB_RUST_PANIC == abort ]]; then
			build_std+=",panic_abort"
		fi

		RUSTFLAGS="${rf[*]}" \
			AR=llvm-ar \
			PKG_CONFIG_ALLOW_CROSS_x86_64_pc_windows_gnu=1 \
			PKG_CONFIG_x86_64_pc_windows_gnu=ab-pkg-config \
			TARGET_CC=ab-clang \
			run cargo "+$toolchain" cinstall \
			$build_std \
			$release \
			--lib \
			--library-type=staticlib \
			--prefix="$AB_PREFIX" \
			--target x86_64-pc-windows-gnu \
			--config "$AB_CARGO_CONFIG" \
			"$@"
	)
}

function filter() {
	set -u
	local a="$1"
	if [[ $# -gt 1 ]]; then
		error "multiple arguments past to filter: $*"
	fi

	if [[ $a == :* ]]; then
		printf '%s' "${a:1}"
		return 0
	fi

	local mode=lib
	if [[ $APP ]]; then
		mode=app
	fi

	if [[ $a == $mode:* ]]; then
		a="${a:$((1 + ${#mode}))}"
	elif [[ $a == app:* || $a == lib:* ]]; then
		return 1
	fi

	if [[ $a == $AB_TOOLCHAIN:* ]]; then
		printf '%s' "${a:$((1 + ${#AB_TOOLCHAIN}))}"
	elif [[ $a == gcc:* || $a == llvm:* ]]; then
		return 1
	else
		printf '%s' "$a"
	fi
}

function set_flags() {
	# Syntax: [lib|app]:[gcc|llvm]:[c|cxx|link|x|p|r]:*
	set -u

	local a
	export CFLAGS="" CXXFLAGS="" CPPFLAGS="" LDFLAGS="" RUSTFLAGS=""

	for a in "${BASE_FLAGS[@]}" "-L$AB_PREFIX/lib" "-I$AB_PREFIX/include" "$@"; do
		if ! a="$(filter "$a")"; then
			continue
		fi

		case "$a" in
		link:*) LDFLAGS+=" $(remove_prefix "$a" link:)" ;;
		c:*) CFLAGS+=" $(remove_prefix "$a" c:)" ;;
		cxx:*) CXXFLAGS+=" $(remove_prefix "$a" cxx:)" ;;
		cpp:*) CPPFLAGS+=" $(remove_prefix "$a" cpp:)" ;;
		x:* | -[sg] | -mtune=* | -march=* | -Wl,* | -Ofast | -O[0123sz] | -Wno-*)
			local flag="$(remove_prefix "$a" x:)"
			CFLAGS+=" $flag"
			CXXFLAGS+=" $flag"
			;;
		p:* | -[ID]*)
			a="$(remove_prefix "$a" p:)"
			CFLAGS+=" $a"
			CXXFLAGS+=" $a"
			CPPFLAGS+=" $a"
			;;
		-D?*)
			CPPFLAGS+=" $a"
			CFLAGS+=" $a"
			CXXFLAGS+=" $a"
			;;
		r:*) RUSTFLAGS+=" $(remove_prefix "$a" r:)" ;;
		-L* | -l* | -flto* | -fno-lto | -static)
			CFLAGS+=" $a"
			CXXFLAGS+=" $a"
			LDFLAGS+=" $a"
			;;
		*)
			error "argument with invalid syntax passed to set_flazgs: $(printf %q "$a")"
			;;
		esac
	done
}

function set_rustflags() {
	set -u
	local a
	export RUSTFLAGS=""
	for a in "${BASE_FLAGS[@]}"; do
		if [[ $a == r:* ]]; then
			RUSTFLAGS+=" $(remove_prefix "$a" r:)"
		fi
	done
	RUSTFLAGS+=" $*"
}

function ensure_rust_target() {
	set -ue

	if [[ $AB_HAVE_RUSTUP_WINDOWS_TARGET == 1 ]]; then
		return
	fi

	local toolchain=nightly
	if [[ $AB_RUST_BUILD_STD == 0 ]]; then
		toolchain=stable
	fi

	verbose "checking if the $toolchain x86_64-pc-windows-gnu target is installed with rustup"
	local output
	output="$(rustup target list --installed --toolchain "$toolchain")"
	local found_target=0

	local s
	while read -r s; do
		if [[ $s == x86_64-pc-windows-gnu ]]; then
			verbose "yes"
			found_target=1
			break
		fi
	done <<<"$output"

	if [[ $found_target == 0 ]]; then
		info "installing the rust target for x86_64-pc-windows-gnu"
		rustup target add x86_64-pc-windows-gnu --toolchain stable
		info "successfully installed the windows target"
	fi

	if [[ $AB_RUST_BUILD_STD != 0 ]]; then
		verbose "checking if the rust-src component is installed for nightly x86_64-pc-windows-gnu"
		local found_rust_src=0
		output="$(rustup component list --installed --toolchain nightly)"
		while read -r s; do
			if [[ $s == rust-src ]]; then
				found_rust_src=1
				verbose "yes"
				break
			fi
		done <<<"$output"

		if [[ $found_rust_src == 0 ]]; then
			info "installing the rust-src component for nightly x86_64-pc-windows-gnu with rustup"
			run rustup component add rust-src --toolchain=nightly --target=x86_64-pc-windows-gnu
		fi
	fi

	AB_HAVE_RUSTUP_WINDOWS_TARGET=1
}

function need_exec() {
	set -u
	local missing=()
	local a
	for a in "$@"; do
		if ! type "$a" &>/dev/null; then
			missing+=("$a")
		fi
	done

	if [[ ${#missing[@]} != 0 ]]; then
		eprint "autobuild: missing required executables on the system: ${missing[*]}"
		exit 5
	fi
}

function apply_patches() {
	local p
	for p in "${PATCHES:?}"/*.patch; do
		run patch "$@" -i "$p" || return
	done
}

function append_pkgconfig() {
	set -u
	local c link a c_arr link_arr
	c="$(x86_64-w64-mingw32-pkg-config --static --cflags "$@")" || return
	link="$(x86_64-w64-mingw32-pkg-config --static --libs "$@")" || return
	split c_arr "$c" " "
	split link_arr "$link" " "

	for a in "${c_arr[@]}" "${link_arr[@]}"; do
		if [[ $a =~ ^(\-L/usr/lib(64)?|\-L/lib(64)?|\-I/usr/include|\-L/usr/local/lib(64)?|\-I/usr/local/include)/?$ ]]; then
			eprint "autobuild error: append_pkgconfig: tried to add include/lib flags pointing to build machine's root: $a"
			error "args: $*"
		fi
	done

	export LDFLAGS+=" $link"
	export CFLAGS+=" $c"
	for a in "${link_arr[@]}"; do
		export CFLAGS+=" -Wl,$a"
		export CXXFLAGS+=" -Wl,$a"
	done
}

function ab-pkg-config() {
	# Sanity check
	local has_flag=0
	local a
	for a in "$@"; do
		if [[ $a == --libs || $a == --cflags ]]; then
			has_flag=1
		fi
	done
	if [[ $has_flag == 0 ]]; then
		error "ab-pkg-config called but no --libs or --cflags flag was specified"
	fi

	x86_64-w64-mingw32-pkg-config "$@"
}

function is_checked() {
	if [[ $# != 1 ]]; then
		error "wrong number of arguments passed to is_checked: $#"
	fi
	test -e "$AB_TMP/checked-for-updates/$1"
}

function add_checked() {
	set -u
	local a
	for a in "$@"; do
		touch "$AB_TMP/checked-for-updates/$a" || return
	done
}

function ab_prefix_db() {
	set -u
	local short="$(remove_prefix "$AB_PREFIX" "$AB_ROOT/")"
	echo "${short//\//.}"
}

function db.set() {
	set -u
	local prefix
	prefix="$(ab_prefix_db)"
	mkdir -p "$AB_DB/$prefix" || return
	local key="$prefix/$1"
	local value="$2"
	printf %s "$value" >"$AB_DB/$key"
}

function db.get() {
	set -u
	local key="$(ab_prefix_db)/$1"
	local p="$AB_DB/$key"
	test -e "$p" || return
	cat "$p"
}

function download() {
	set -u
	local recipe="$1"
	local test="${2:-}"
	if [[ ! $APP && -z $test ]]; then
		error "path to test for not provided to download"
	fi
	# Sanity check.
	if [[ ! $recipe =~ ^[a-zA-Z0-9_\-]+$ ]]; then
		error "recipe name contains illegal characters: $recipe"
	elif [[ ! (-f "$AB_DIR/lib-recipes/$recipe" || -f "$AB_DIR/multi-recipes/$recipe" || -f "$AB_DIR/app-recipes/$recipe") ]]; then
		error "no build script found for $recipe"
	fi
	local args=("${@:3}")

	local val
	if [[ -n ${AB_REPOS_SHIM["$recipe"]:-} ]]; then
		recipe="${AB_REPOS_SHIM["$recipe"]}"
	fi
	# shellcheck disable=SC2034
	PATCHES="$AB_DIR/patches/$recipe"
	# Try in order
	local val
	val="${AB_REPOS_DIRECT["$recipe"]:-}"
	if [[ -n $val ]]; then
		_download_direct "$recipe" "$val" "$test" "${args[@]}"
		return
	fi

	val="${AB_REPOS_HTTP["$recipe"]:-}"
	if [[ -n $val ]]; then
		local site="${val%%:*}"
		if [[ -z $site ]]; then
			error "$recipe: source has an empty site prefix in \$AB_REPOS_HTTP: $val"
		elif [[ $site == http || $site == https ]]; then
			error "$recipe: source does not have a site prefix in \$AB_REPOS_HTTP: $val"
		fi

		val="${val:((1 + ${#site}))}"
		if [[ $site == msys ]]; then
			local i
			val="$val --$AB_MINGW_CRT"
			case "$AB_MINGW_CRT" in
			msvcrt)
				if [[ ${#args[@]} == 0 ]]; then
					args=(mingw64)
				else
					for ((i = 0; i < ${#args[@]}; i++)); do
						args[i]="mingw64/${args[i]}"
					done
				fi
				;;
			ucrt)
				if [[ ${#args[@]} == 0 ]]; then
					args=(ucrt64)
				else
					for ((i = 0; i < ${#args[@]}; i++)); do
						args[i]="ucrt64/${args[i]}"
					done
				fi
				;;
			"") error "\$AB_MINGW_CRT not set" ;;
			*) error "unknown value $AB_MINGW_CRT for \$AB_MINGW_CRT" ;;
			esac
		fi

		_download_http "$site" "$recipe" "$val" "$test" "${args[@]}"
		return
	fi

	val="${AB_REPOS_GIT["$recipe"]:-}"
	if [[ -n $val ]]; then
		if [[ $val == master:* ]]; then
			_clone_master "$recipe" "${val:7}" "$test" "${args[@]}"
		else
			_clone_repo "$recipe" "$val" "$test" "${args[@]}"
		fi
		return
	fi

	error "download source not set for $recipe"
}

function _download_http() {
	set -ue
	# Clear AB_LIB_FORCE so only the top-level library is affected.
	local _force="$AB_LIB_FORCE"
	AB_LIB_FORCE=""

	local version_check="$1"
	local recipe="$2"
	local name="$3"
	local test="${4:-}"
	local extract="${5:-}"
	if [[ -z $test && ! $APP ]]; then
		error "path to test for (argument 4) not provided to download"
	fi

	cd "$AB_SOURCES"
	if is_checked "$recipe"; then
		verbose "already checked for updates to $recipe"
		if [[ ! $APP && -e "$AB_PREFIX"/$test ]]; then
			info "$recipe is up to date"
			exit 0
		fi
		cd "$recipe"
		git clean -qffxd
		git restore . --quiet
		git reset --hard --quiet
		AB_TAG="$(cat .ab_version)"
		return
	fi

	local installed_version="$(db.get "$recipe" || true)"
	local output url version
	split name "$name" " "
	output="$(ab-helper check "$version_check" "${name[@]}")"
	# Newlines are not split properly with split.
	output="${output//$'\n'/;}"
	local arr=()
	split arr "$output" ";"
	url="${arr[0]:-}"
	version="${arr[1]:-}"
	if [[ -z $url ]]; then
		error "could not check for updates: python script printed nothing"
	fi
	export AB_TAG="$version"
	LIB_ID="$version"
	add_checked "$recipe"

	if [[ $installed_version == "$version" ]]; then
		if [[ ! $APP && ! $_force && -e "$AB_PREFIX"/$test ]]; then
			info "$recipe is up to date"
			exit 0
		elif [[ -d $recipe ]]; then
			cd "$recipe"
			reset_repo
			return
		fi
	fi

	# Is it downloaded but not installed?
	if [[ -e "./$recipe/.ab_version" ]]; then
		local downloaded_version
		downloaded_version="$(cat "./$recipe/.ab_version")"
		if [[ -n $version && $downloaded_version == "$version" ]]; then
			# No need to download the same thing.
			cd "$recipe"
			reset_repo
			return
		fi
	fi

	# Source needs updating.
	rm -rf "./$recipe"
	mkdir "$recipe"
	cd "$recipe"

	_wget_extract "$url" "$extract"
	printf %s "$version" >.ab_version
	git init . -q
	git add .
	# Git requires name and email.
	git commit --author="autobuild script <autobuild@invalid.invalid>" -qm autobuild-init
}

function _download_direct() {
	set -ue
	# Clear AB_LIB_FORCE so only the top-level library is affected.
	local _force="$AB_LIB_FORCE"
	AB_LIB_FORCE=""

	local recipe="$1"
	local url="$2"
	local test="${3:-}"
	local extract="${4:-}"
	if [[ ! $APP && -z $test ]]; then
		error "missing required argument: path to test for"
	fi

	local id="${url%%::*}"
	if [[ $id != "$url" ]]; then
		url="${url:${#id}+2}"
		if [[ -z $url ]]; then
			error "url in \$AB_REPOS_DIRECT for $recipe is empty"
		elif [[ -z $id ]]; then
			error "version string set in \$AB_REPOS_DIRECT is empty for $recipe"
		fi
		AB_TAG="$id"
	fi

	LIB_ID="$id"

	local installed_id downloaded_id
	cd "$AB_SOURCES"

	installed_id="$(db.get "$recipe" || true)"
	downloaded_id="$(cat 2>/dev/null "$recipe/.ab_id" || true)"

	if [[ -e $recipe && $downloaded_id != "$id" ]]; then
		info "detected change, removing existing download"
		rm -rf "./$recipe"
	elif [[ ! $APP && ! $_force && $installed_id == "$id" && -e "$AB_PREFIX"/$test ]]; then
		info "$recipe is up to date"
		exit 0
	elif [[ -e $recipe ]]; then
		cd "$recipe"
		reset_repo
		return
	fi

	mkdir "$recipe"
	cd "$recipe"
	_wget_extract "$url" "$extract"
	check_version_script "$recipe"
	printf %s "$LIB_ID" >.ab_id
	git init . -q
	git add .
	git commit --author="autobuild script <autobuild@invalid.invalid>" -qm autobuild-init
	# If it's an app, ensure that a version was specified or determined through a version script.
	if [[ $APP && (-z $id && -z $VERSION_SCRIPT_OUTPUT) ]]; then
		error "version info not set for $recipe: apps need version information provided or determined via a version script if the source is set to a direct download"
	fi
}

function _clone_repo() {
	set -ue
	# Clear AB_LIB_FORCE so only the top-level library is affected.
	local _force="$AB_LIB_FORCE"
	AB_LIB_FORCE=""

	local name="$1"
	local arr=()
	split arr "$2" " "
	local git="${arr[0]}"
	# $args is passed to latest_git_version
	local args=("${arr[@]:1}")
	local test="${3:-}"
	if [[ -z $APP && -z $test ]]; then
		error "path to test for not provided"
	fi

	local current_tag="" installed_tag=""
	installed_tag="$(db.get "$name" || true)"
	local main_branch=""
	cd -- "$AB_SOURCES"

	if is_checked "$name"; then
		# We checked for updates for this one already.
		verbose "already checked for updates to $name"
		current_tag="$(git -C "$name" describe --tags --abbrev=0)"
		if [[ ! $_force && ! $APP && -e "$AB_PREFIX"/$test && $installed_tag == "$current_tag" ]]; then
			info "$name is up to date"
			exit 0
		fi
		LIB_ID="$current_tag"
		AB_TAG="$current_tag"
		check_version_script "$name"
		cd "$name"
		reset_repo
		return
	fi

	if [[ -e $name ]]; then
		cd -- "$name"
		current_tag="$(latest_git_tag "${args[@]}")"
		local main_branch
		main_branch="$(git symbolic-ref refs/remotes/origin/HEAD | sed 's@^refs/remotes/origin/@@')"
		if [[ -z $main_branch ]]; then
			eprint "error: cannot determine the current branch"
			return 1
		fi

		reset_repo
		git checkout --quiet "$main_branch"
		git pull --quiet
		git submodule update --recursive --remote --quiet
	else
		info "cloning $name"
		git clone --quiet --recursive "$git" "$name"
		cd -- "$name"
	fi

	local latest_tag
	latest_tag="$(latest_git_tag "${args[@]}")"
	git checkout --quiet "$latest_tag"
	add_checked "$name"
	AB_TAG="$latest_tag"
	LIB_ID="$latest_tag"
	check_version_script "$name"

	if [[ 
		-z $APP &&
		! $_force &&
		((-n $VERSION_SCRIPT_OUTPUT && $LIB_ID == "$installed_tag") ||
		(-z $VERSION_SCRIPT_OUTPUT && $installed_tag == "$latest_tag" && $current_tag == "$latest_tag")) &&
		-e "$AB_PREFIX"/$test ]]; then
		info "$name is up to date"
		exit 0
	fi
}

function msys_install() (
	set -ue

	eprint "+ msys_install"
	local f
	shopt -s globstar nullglob
	# Sanity check
	if [[ ! (-d lib || -d include || -d share) ]]; then
		error "no lib, include or share directories exist"
	fi

	for f in **/*.pc; do
		edit-pc -p "$f" "\$prefix=$AB_PREFIX"
	done
	for f in **/*.la; do
		ab-helper edit-lt -p "$f" --relocate "$AB_PREFIX"
	done

	# rm -f ./**/*.dll.a
	for f in lib include share; do
		if [[ -d $f ]]; then
			# rsync --checksum -a "$f/" "$AB_PREFIX/$f/"
			ab-helper merge-dir --exclude '*.dll.a' "$f" "$AB_PREFIX/$f/"
		fi
	done
)

function _wget_extract() (
	set -ue
	# need_exec wget
	shopt -s dotglob nullglob
	set -o pipefail
	local url="$1"
	local extract="${2:-}"
	local save_as=""
	if [[ $url =~ \#[^/]+$ ]]; then
		url="${url%"#"*}"
		save_as="${url##*"#"}"
		# Sanity check, we really don't want to be working with files outside the current directory.
		if [[ $save_as == /* || $save_as == *..* ]]; then
			error "dangerous url fragment to save as: $save_as"
		fi
	fi
	# Some more sanity checks.
	if [[ -n $extract && ($extract == /* || $extract == *..*) ]]; then
		error "dangerous path provided as extract: $extract"
	fi

	info "downloading $url"
	# wget -nv -O- "$url" | bsdtar -x
	ab-helper dl -O- "$url" | bsdtar -x

	local files=(*)

	if [[ ${#files[@]} == 0 ]]; then
		eprint "error: failed to download/extract $recipe; there are no files"
		exit 1
	elif [[ ${#files[@]} == 1 ]]; then
		# Unnest one level.
		local f="${files[0]}"
		mv "./$f"/* .
		rmdir "./$f"
	fi
	# Remove other files if $extract is set.
	if [[ -n $extract ]]; then
		local parent="${extract%%/*}"
		for f in *; do
			if [[ $f != "$parent" ]]; then
				rm -rf "./$f"
			fi
		done
		# And unnest so that files in $extract are at the top level
		f="$(echo *)"
		mv "./$extract"/* .
		rm -rf "./$f"
	fi
)

function ab_get_version() {
	set -u
	local name="${1:?autobuild error: missing requirement argument 'name'  to ab_get_version}"
	if [[ $# != 1 ]]; then
		error "too many arguments passed to ab_get_version"
	fi
	if [[ -f "$AB_SOURCES/$name/.ab_version" ]]; then
		cat "$AB_SOURCES/$name/.ab_version"
	else
		git -C "$AB_SOURCES/$name" describe --tags --abbrev=0
	fi
}

function check_version_script() {
	set -ue
	# This needs to be non-local
	VERSION_SCRIPT_OUTPUT=""
	local name="$1"
	if [[ -f "$AB_DIR/version-scripts/$name" ]]; then
		verbose "have a version script for $name; invoking..."
		local res
		# This needs to be non-local so other scopes can read it.
		res="$(
			cd -- "$AB_SOURCES/$name"
			# shellcheck disable=SC1090
			. "$AB_DIR/version-scripts/$name"
		)"
		if [[ -z $res ]]; then
			verbose "version script printed nothing"
		else
			verbose "version script output: $res"
			VERSION_SCRIPT_OUTPUT="$res"
			AB_TAG="$res"
			LIB_ID="$res"
		fi
	fi
}

function edit-pc() {
	ab-helper edit-pc "$@"
}

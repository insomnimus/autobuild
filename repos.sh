#!/usr/bin/env bash

declare -A AB_REPOS_GIT=(
	[faust]=https://github.com/grame-cncm/faust.git
	["cargo-binutils"]=https://github.com/rust-embedded/cargo-binutils.git
	["cargo-clone"]=https://github.com/JanLikar/cargo-clone
	["cargo-edit"]=https://github.com/killercup/cargo-edit
	["cargo-machete"]=https://github.com/bnjbvr/cargo-machete.git
	[catbox]=https://github.com/Savolae/catbox
	[cross]=https://github.com/cross-rs/cross.git
	[miniserve]=https://github.com/svenstaro/miniserve.git
	[ripgrep]=https://github.com/BurntSushi/ripgrep
	[ruff]=https://github.com/astral-sh/ruff.git
	[rustfilt]=https://github.com/luser/rustfilt.git
	[sd]=https://github.com/chmln/sd.git
	[tokei]=https://github.com/XAMPPRocky/tokei
	[uv]=https://github.com/astral-sh/uv
	[mdbook]=https://github.com/rust-lang/mdBook.git
	# [luajit]=master:https://luajit.org/git/luajit.git
	[luajit]=master:https://github.com/LuaJIT/LuaJIT.git
	[tidy]=https://github.com/htacg/tidy-html5
	["rust-bindgen"]=https://github.com/rust-lang/rust-bindgen
	# [srgn]=https://github.com/alexpovel/srgn.git
	["vorbis-tools"]=https://github.com/xiph/vorbis-tools
	[mpc]=https://github.com/MusicPlayerDaemon/mpc
	[xz]="https://github.com/tukaani-project/xz sort=version"
	[flac]=https://gitlab.xiph.org/xiph/flac
	[ffmpeg]=https://github.com/FFmpeg/FFmpeg
	[mpv]=https://github.com/mpv-player/mpv
	# [fluidsynth]=https://github.com/FluidSynth/fluidsynth
	[jq]=https://github.com/jqlang/jq
	[oniguruma]=https://github.com/kkos/oniguruma.git
	[aria2]=https://github.com/aria2/aria2
	[mujs]=git://git.ghostscript.com/mujs.git
	[bzip2]=master:https://gitlab.com/bzip2/bzip2
	[vorbis]=https://gitlab.xiph.org/xiph/vorbis.git
	[ao]=https://gitlab.xiph.org/xiph/libao
	[ogg]=https://gitlab.xiph.org/xiph/ogg
	[mpdclient]=https://github.com/MusicPlayerDaemon/libmpdclient
	[nfs]=https://github.com/sahlberg/libnfs
	[adplug]=https://github.com/adplug/adplug
	[binio]=https://github.com/adplug/libbinio
	[samplerate]=https://github.com/libsndfile/libsamplerate
	[ffnvcodec]=https://github.com/FFmpeg/nv-codec-headers
	[opus]=https://github.com/xiph/opus
	[aom]=https://aomedia.googlesource.com/aom
	[vpx]=https://github.com/webmproject/libvpx
	[dav1d]=https://code.videolan.org/videolan/dav1d
	["fdk-aac"]=https://github.com/mstorsjo/fdk-aac
	[x264]=master:https://code.videolan.org/videolan/x264.git
	[x265]=https://bitbucket.org/multicoreware/x265_git.git
	["svt-hevc"]=https://github.com/OpenVisualCloud/SVT-HEVC
	[fribidi]=https://github.com/fribidi/fribidi
	[unibreak]=https://github.com/adah1972/libunibreak
	# [fontconfig]=https://gitlab.freedesktop.org/fontconfig/fontconfig
	[freetype]=https://gitlab.freedesktop.org/freetype/freetype.git
	[harfbuzz]=https://github.com/harfbuzz/harfbuzz
	[mysofa]=https://github.com/hoene/libmysofa
	[snappy]=https://github.com/google/snappy
	[soxr]=https://git.code.sf.net/p/soxr/code
	[tiff]=https://gitlab.com/libtiff/libtiff
	[deflate]=https://github.com/ebiggers/libdeflate
	[zstd]=https://github.com/facebook/zstd
	[lerc]=https://github.com/Esri/lerc
	[webp]=https://chromium.googlesource.com/webm/libwebp
	[bluray]=https://code.videolan.org/videolan/libbluray.git
	[sndfile]=https://github.com/libsndfile/libsndfile
	[chromaprint]=https://github.com/acoustid/chromaprint
	[id3tag]=https://codeberg.org/tenacityteam/libid3tag
	[fluidsynth]=https://github.com/FluidSynth/fluidsynth
	[ssh2]=https://github.com/libssh2/libssh2
	[fido2]=https://github.com/Yubico/libfido2
	[cbor]=https://github.com/pjk/libcbor
	[cjson]=https://github.com/DaveGamble/cJSON
	[hidapi]=https://github.com/libusb/hidapi
	["p11-kit"]=https://github.com/p11-glue/p11-kit
	[speex]=https://gitlab.xiph.org/xiph/speex
	[theora]=https://gitlab.xiph.org/xiph/theora
	[vidstab]=https://github.com/georgmartius/vid.stab
	[zimg]=https://github.com/sekrit-twc/zimg
	[avisynth]=https://github.com/AviSynth/AviSynthPlus
	[jasper]=https://github.com/jasper-software/jasper
	[heif]=https://github.com/strukturag/libheif
	[srt]=https://github.com/Haivision/srt
	[vmaf]=https://github.com/Netflix/vmaf
	["svt-av1"]=https://gitlab.com/AOMediaCodec/SVT-AV1
	["vulkan-headers"]=https://github.com/KhronosGroup/Vulkan-Hpp
	[vulkan]=https://github.com/KhronosGroup/Vulkan-Loader
	[placebo]="https://code.videolan.org/videolan/libplacebo sort=version"
	[xxhash]=https://github.com/Cyan4973/xxHash
	[lcms2]=https://github.com/mm2/Little-CMS
	["spirv-headers"]=https://github.com/KhronosGroup/SPIRV-Headers
	# ["spirv-tools"]=https://github.com/KhronosGroup/SPIRV-Tools
	["spirv-cross"]=https://github.com/KhronosGroup/SPIRV-Cross
	[shaderc]=https://github.com/google/shaderc
	[rav1e]=https://github.com/xiph/rav1e
	[uchardet]=https://gitlab.freedesktop.org/uchardet/uchardet
	[blake2]=https://github.com/BLAKE2/libb2
	[pcre2]=https://github.com/PhilipHazel/pcre2
	["c-ares"]=https://github.com/c-ares/c-ares
	[psl]=https://github.com/rockdaboot/libpsl
	[instpatch]=https://github.com/swami/libinstpatch
	[libuv]=https://github.com/libuv/libuv
	["jpeg-turbo"]=https://github.com/libjpeg-turbo/libjpeg-turbo
	["decklink-headers"]=master:https://gitlab.com/m-ab-s/decklink-headers
	[tre]=master:https://github.com/laurikari/tre
	[rist]=https://code.videolan.org/rist/librist
	[jxl]=https://github.com/libjxl/libjxl
	[dlfcn]=https://github.com/dlfcn-win32/dlfcn-win32
	[sox]=master:https://git.code.sf.net/p/sox/code
	["espeak-ng"]=https://github.com/espeak-ng/espeak-ng
	[zlib]=https://github.com/zlib-ng/zlib-ng
	[hsts]=https://gitlab.com/rockdaboot/libhsts
	[xevd]=master:https://github.com/mpeg5/xevd
	[xeve]=master:https://github.com/mpeg5/xeve
	[tlrc]=https://github.com/tldr-pages/tlrc
	[pixz]=master:https://github.com/vasi/pixz.git
	[kvazaar]=https://github.com/ultravideo/kvazaar.git
	[modplug]=master:https://github.com/Konstanty/libmodplug.git
	[xavs2]=https://github.com/pkuvcl/xavs2.git
	[davs2]=https://github.com/pkuvcl/davs2.git
	[uavs3d]=master:https://github.com/uavs3/uavs3d.git
	[vvenc]=https://github.com/fraunhoferhhi/vvenc.git
	[vvdec]=https://github.com/fraunhoferhhi/vvdec.git
	[lc3]=https://github.com/google/liblc3.git
	["opus-tools"]=master:https://github.com/xiph/opus-tools.git
	[opusenc]=https://github.com/xiph/libopusenc.git

	# Go apps
	[actionlint]=https://github.com/rhysd/actionlint.git
	[gh]=https://github.com/cli/cli.git
	[gofumpt]=https://github.com/mvdan/gofumpt.git
	[shfmt]=https://github.com/mvdan/sh.git
	[staticcheck]=https://github.com/dominikh/go-tools.git
	[yq]=https://github.com/mikefarah/yq.git
	[usql]="https://github.com/xo/usql.git sort=version"
	["fluidsynth-vst"]=master:https://github.com/AZSlow3/FluidSynthVST.git
)

declare -A AB_REPOS_DIRECT=(
	["spirv-tools"]=1.4.309::https://github.com/KhronosGroup/SPIRV-Tools/archive/vulkan-sdk-1.4.309.0/spirv-tools-1.4.309.0.tar.gz
	[sqlite3]=3.45.2::https://www.sqlite.org/2024/sqlite-src-3450200.zip
	[sdl2]=2.32.4::https://github.com/libsdl-org/SDL/releases/download/release-2.32.4/SDL2-2.32.4.tar.gz
	[fontconfig]=2.15.0::https://www.freedesktop.org/software/fontconfig/release/fontconfig-2.15.0.tar.xz
	[libressl]=3.1.1::https://ftp.openbsd.org/pub/OpenBSD/LibreSSL/libressl-3.1.1.tar.gz
	[winpthreads]=v6.0.1::https://sourceforge.net/projects/mingw-w64/files/mingw-w64/mingw-w64-release/mingw-w64-v6.0.1.tar.bz2/download
	# TODO: Change spirv-* stuff back to git once the 2024 version stabilizes (currently the latest git tags are incompatible)
	# ["spirv-tools"]=1.3.275.0::https://github.com/KhronosGroup/SPIRV-Tools/archive/refs/tags/vulkan-sdk-1.3.275.0.tar.gz
	# ["spirv-headers"]=1.3.275.0::https://github.com/KhronosGroup/SPIRV-Headers/archive/refs/tags/vulkan-sdk-1.3.275.0.tar.gz
	# ["spirv-cross"]=1.3.275.0::https://github.com/KhronosGroup/SPIRV-Cross/archive/refs/tags/vulkan-sdk-1.3.275.0.tar.gz
	[tcl]=https://downloads.sourceforge.net/sourceforge/tcl/tcl8.6.13-src.tar.gz
	[openldap]=https://www.openldap.org/software/download/OpenLDAP/openldap-release/openldap-2.6.6.tgz
	[kerberos]=https://kerberos.org/dist/krb5/1.21/krb5-1.21.2.tar.gz
	[jpeg]=http://ijg.org/files/jpegsr9e.zip
	[gnurx]=https://downloads.sourceforge.net/mingw/Other/UserContributed/regex/mingw-regex-2.5.1/mingw-libgnurx-2.5.1-src.tar.gz
	[fftw]=https://fftw.org/fftw-3.3.10.tar.gz
	[gsm]=https://www.quut.com/gsm/gsm-1.0.22.tar.gz
	[rubberband]=https://breakfastquay.com/files/releases/rubberband-3.3.0.tar.bz2
	[mad]=https://downloads.sourceforge.net/sourceforge/mad/libmad-0.15.1b.tar.gz
	[ffmpeg6]=6.1.1::https://github.com/FFmpeg/FFmpeg/archive/refs/tags/n6.1.1.tar.gz
	[ffnvcodec6]=n12.2.72.0::https://github.com/FFmpeg/nv-codec-headers/archive/refs/tags/n12.1.14.0.tar.gz
)

declare -A AB_REPOS_HTTP=(
	# Single projects
	# [sqlite3]=sqlite:--
	# [boost]=boost:
	["vst3-sdk"]=vst3-sdk:

	#MSYS2 binary packages
	[boost]=msys:boost
	[icu]=msys:icu
	[id3lib]=msys:id3lib
	[idn2]=msys:libidn2
	[tasn1]=msys:libtasn1
	[openmp]=msys:llvm-openmp
	[readline]=msys:readline
	[systre]=msys:libsystre
	[termcap]=msys:termcap

	# OpenBSD release archives
	# [libressl]=openbsd:'LibreSSL libressl'

	# Gnome release archives
	[glib]=gnome:glib
	[xml2]=gnome:libxml2

	# GNU release archives
	[ltdl]=gnu:libtool
	# [tasn1]=gnu:libtasn1
	# [idn2]=gnu:'libidn libidn2'
	[iconv]=gnu:libiconv
	[libffi]=gnu:https://sourceware.org/pub/libffi
	[cdio]=gnu:libcdio
	[gmp]=gnu:gmp
	[unistring]=gnu:libunistring
	[wget]=gnu:wget
	[wget2]=gnu:'wget wget2'
	[nettle]=gnu:nettle
	[sed]=gnu:sed
	["cdio-paranoia"]=gnu:'libcdio libcdio-paranoia'

	# GnuPG source releases
	[gcrypt]=gnupg:libgcrypt
	[gpgme]=gnupg:gpgme
	[assuan]=gnupg:libassuan
	[gnutls]=gnupg:gnutls
	["gpg-error"]=gnupg:'gpgrt libgpg-error'

	# Sourceforge releases
	[bs2b]=sf:'bs2b libbs2b --dir=libbs2b'
	[cddb]=sf:libcddb
	[giflib]=sf:'giflib -d!*'
	[lame]=sf:lame
	[mng]=sf:'libmng --dir=libmng-devel'
	[mpg123]=sf:mpg123
	["opencore-amr"]=sf:opencore-amr
	[png]=sf:'libpng -dlibpng*'
	[twolame]=sf:twolame

	# Github releases
	[amf]=gh:GPUOpen-LibrariesAndSDKs/AMF
	[archive]=gh:'libarchive/libarchive libarchive'
	[aribb24]=gh:nkoriyama/aribb24
	[ass]=gh:'libass/libass libass'
	[brotli]=gh:google/brotli
	[caca]=gh:'cacalabs/libcaca libcaca'
	[curl]=gh:'curl/curl curl'
	[expat]=gh:'libexpat/libexpat expat'
	[flite]=gh:festvox/flite
	[fluidsynth]=gh:FluidSynth/fluidsynth
	[glslang]=gh:KhronosGroup/glslang
	[gme]=gh:libgme/game-music-emu
	[ilbc]=gh:'TimothyGu/libilbc libilbc'
	[leptonica]=gh:'DanBloomberg/leptonica leptonica'
	[lz4]=gh:'lz4/lz4 lz4'
	[nghttp2]=gh:'nghttp2/nghttp2 nghttp2'
	[nghttp3]=gh:'ngtcp2/nghttp3 nghttp3'
	[ngtcp2]=gh:'ngtcp2/ngtcp2 ngtcp2'
	[opencl]=gh:KhronosGroup/OpenCL-ICD-Loader
	["opencl-headers"]=gh:KhronosGroup/OpenCL-Headers
	[openjpeg]=gh:uclouvain/openjpeg
	[opusfile]=gh:'xiph/opusfile opusfile'
	[pcaudiolib]=gh:'espeak-ng/pcaudiolib pcaudiolib'
	[piper]=gh:rhasspy/piper
	["piper-phonemize"]=gh:rhasspy/piper-phonemize
	[sdl3]=gh:'libsdl-org/SDL SDL3'
	[srgn]=gh:alexpovel/srgn
	[upx]=gh:'upx/upx -r ^upx-<version>-src<ext>$'
	[vpl]=gh:intel/libvpl
	[wavpack]=gh:'dbry/WavPack wavpack'
	[zmq]=gh:'zeromq/libzmq zeromq'
)

declare -A AB_REPOS_SHIM=(
	["curl-all"]=curl

	["curl-gnutls"]=curl

	["curl-libressl"]=curl

	["curl-winssl"]=curl
)

# Cmake toolchain file for 64 bit windows

set(CMAKE_SYSTEM_PROCESSOR AMD64)
set(CMAKE_SYSTEM_NAME Windows)

set(CMAKE_C_COMPILER ab-clang)
set(CMAKE_CXX_COMPILER ab-clang++)
set(CMAKE_AR llvm-ar)
set(CMAKE_STRIP ab-llvm-strip)
set(CMAKE_NM llvm-nm)
set(CMAKE_RANLIB llvm-ranlib)
set(CMAKE_RC_COMPILER x86_64-w64-mingw32-windres)
set(CMAKE_C_COMPILER_TARGET x86_64-w64-mingw32)
set(CMAKE_CXX_COMPILER_TARGET x86_64-w64-mingw32)

find_program(PKG_CONFIG_EXECUTABLE x86_64-w64-mingw32-pkg-config)
find_program(CMAKE_RC_COMPILER NAMES ${CMAKE_RC_COMPILER})
find_program(CMAKE_C_COMPILER NAMES ${CMAKE_C_COMPILER})
find_program(CMAKE_CXX_COMPILER NAMES ${CMAKE_CXX_COMPILER})

# Do not use build environment's programs
# set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)

# Note: do not set to ONLY!
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY BOTH)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE BOTH)

# This is so winpthreads is statically linked
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -static")

# Suffix of libraries and stuff
set(CMAKE_LINK_LIBRARY_SUFFIX ".a")
set(CMAKE_STATIC_LIBRARY_SUFFIX ".a")
set(CMAKE_FIND_LIBRARY_SUFFIXES .a .lib)
set(CMAKE_FIND_LIBRARY_PREFIXES lib)

# Do not use system libraries
set(CMAKE_IGNORE_PATH "/lib:/lib64:/usr/lib:/usr/lib64:/usr/local/lib:/usr/local/lib64")

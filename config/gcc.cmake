# Cmake toolchain file for 64 bit windows

set(CMAKE_SYSTEM_PROCESSOR AMD64)
set(CMAKE_SYSTEM_NAME Windows)
set(CMAKE_C_COMPILER x86_64-w64-mingw32-gcc)
set(CMAKE_CXX_COMPILER x86_64-w64-mingw32-g++)
set(CMAKE_AR x86_64-w64-mingw32-gcc-ar)
set(CMAKE_STRIP x86_64-w64-mingw32-strip)
set(CMAKE_NM x86_64-w64-mingw32-gcc-nm)
set(CMAKE_RANLIB x86_64-w64-mingw32-gcc-ranlib)
set(CMAKE_RC_COMPILER x86_64-w64-mingw32-windres)

# Do not use build environment's programs
# set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)

# Note: do not set to ONLY!
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY BOTH)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE BOTH)

# This is so winpthreads is statically linked
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -static")
set(CMAKE_SHARED_LINKER_FLAGS "${CMAKE_SHARED_LINKER_FLAGS} -static")
# set(CMAKE_MODULE_LINKER_FLAGS "${CMAKE_MODULE_LINKER_FLAGS} -static")

# Optimization flags
set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} -O2")
set(CMAKE_C_FLAGS_RELEASE "${CMAKE_C_FLAGS_RELEASE} -O2")
# Suffix of libraries and stuff
set(CMAKE_LINK_LIBRARY_SUFFIX ".a")
set(CMAKE_STATIC_LIBRARY_SUFFIX ".a")
set(CMAKE_FIND_LIBRARY_SUFFIXES .a .lib)
set(CMAKE_FIND_LIBRARY_PREFIXES lib)

# Do not use system libraries
set(CMAKE_IGNORE_PATH "/lib:/lib64:/usr/lib:/usr/lib64:/usr/local/lib:/usr/local/lib64")

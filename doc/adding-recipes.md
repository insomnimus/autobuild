# Adding New Recipes
Briefly, you add the project source (git URL, direct download URL or other) in the file `repos.sh`; then you create the recipe script in `app-recipes`, `lib-recipes` or `multi-recipes`.

The script must have the same name as the recipe. It also needs to be a Bash script.

The driver will "dot source" the script in a subshell. The script will start with the library already included (`lib.sh`) and the shell options `set -e` and `set -u` set, meaning errors will cause it to exit the script, and undefined variable names will cause errors.

The script is responsible for setting up environment variables related to cross compiling since these vary per project. Refer to the `set_env` function's documentation in [Library Functions](library.md).

## Download Sources
There are 3 types of download sources. Each one of these has a map variable declared in `repos.sh`.
The map keys are the recipe names and the values are information about where to get the source code and how to check for updates.

These 3 are:
- Git sources (`AB_REPOS_GIT`)
- Direct downloads (`AB_REPOS_DIRECT`)
- Projects from known non-git sites (`AB_REPOS_HTTP`)

### `AB_REPOS_GIT`
The values in this map are most often simply URLs to `git clone`.

Normally the latest git tag (obtained with `git describe --tags --abbrev=0`) will be checked out. The git tag is also used as the recipe's version.

You can specify that the default branch (usually `master` or `main`, it's auto detected) be checked out instead by prefixing the url with `master:`.
In this case the recipe will not have a version, which is not allowed for recipes that build apps (app and multi recipes), unless there's a version script available (discussed later in this page).

Finally you can alter the tag detection by adding extra arguments to the value (after spaces):
- `sort=version` Will Find the greatest tag name, treating tag names as version strings.
- `sort=date` will select the tag that's the most recent.

### `AB_REPOS_DIRECT`
This map contains direct downloads; no update checks can be done.

The values in it are download URLs, optionally prefixed with `<version>::` (e.g. `1.2.0::https://example.com/foo.tar.gz`).

Recipes that build apps cannot be put in this map unless they have a version prefix, or have a version script.

### `AB_REPOS_HTTP`
This map is for projects in sites that are known to `ab-helper`.

These are checked for updates using `ab-helper check`.
Please read the documentation on the `check` subcommand in [The Helper Utility](ab-helper.md).

The values in this map are in the form `<site>:<args>, where
- `<site>` is the name of a site known to `ab-helper check`
- And `<args>` are arguments to pass to `ab-helper check <site>` (the arguments are split on spaces)

## The Anatomy of a Recipe Script
The script will usually consist of 3 or 4stages:
1. Download / update source code using the `download` function.
2. Install any dependencies using the `install_lib` function.
3. Build (and often install) the project using functions such as `cmake_install`, `configure_install_par`, `cargo_build`, `meson_build` among others.
4. (For apps and multi recipes) Call the `out` function to "yield" executables.

So the template is something like below.
```bash
#!/bin/env bash

# For lib or multi recipes:
download foo lib/libfoo.a # The script won't progress further if we're up to date and $AB_PREFIX/lib/libfoo.a exists

# If we have dependencies, install them early on
install_lib iconv zlib

# You almost always have to call `set_env` for cross compilation
# This sets env variables such as CC, CXX, CFLAGS etc
set_env

# Since `download` puts the script's working directory to the root of the downloaded source tree, we can just build!
configure_install_par --enable-zlib

# Emit the output of the recipe if it's supposed to be an app or multi recipe
# `out` won't do anything if we're being told to build a library so no need to run it conditionally
# Also you don't need to call it if the recipe's purely a library recipe
out "$AB_PREFIX/bin/foo.exe"
```

## Version Scripts
Some edge cases will require custom version detecting logic; for example Luajit no longer uses git tags or releases; the users are supposed to build from the master branch.
For these cases, you can write a script to determine the version.

These "version scripts" are located in the `version-scripts` directory. They can be written in any language in the autobuild tree.
The script will be invoked from within a downloaded source tree and can mutate the source tree (e.g. run `git checkout`).

The script has to print the version to standard output; any other message (logging/debugging etc) has to be supressed or written to standard error (stderr).

If the script exits with a non-zero exit code, the build is aborted with an error.

## Patches
You can store patches in the `patches/<recipe>` directory under the Autobuild tree per recipe.

In fact you can put any file in there, doesn't need to be patch files.

Scripts will have the `PATCHES` variable set to the path `$AB_DIR/patches/$RECIPE`.

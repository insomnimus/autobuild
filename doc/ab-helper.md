# The Helper Program
Autobuild has a helper program written in Rust.
It's used to download files, check for updates on non-git sources (through web scraping), detect the default C runtime and more.

The program is automatically built and installed into `$AB_ROOT/local/bin` if it's missing; if iT's not missing but outdated, it's also built.

It has a couple subcommands. These are documented in this page.

## `ab-helper check`
Crawls the web for the latest version of a project, as well as fetching the download URL for the release.

If successful, it will print to the standard output exactly 2 lines: first, the download URL; then the version string.

There are a couple of known sites. They're specified as subcommands.

#### Conventions
- Options described as being regular expressions will get processed so that the substrings `<version>` and `<ext>` are replaced with a more elaborate regular expression:
	- `<version>` Will be replaced by a regex that tries to match version strings.
	- `<ext>`: Will be replaced with a regex that tries to match a tar file name extension, including the dot; e.g. it will match `.tar.gz`, `.tgz`, `.tar.zst`, `.txz` and so on.
- Options and arguments described as "filename prefix" are a little complicated. They will be stripped from candidate file names, and then:
	1. If a candidate ends with a tar extension (`.tar.gz`, `.tar.xz`, etc), the extension is stripped; otherwise the candidate is eliminated.
	2. If the candidate starts with the "filename prefix" given by the user, it's stripped; otherwise it's eliminated.
	3. The remaining string now must start with `-`; if so it's trimmed, if not, it's eliminated.
	4. Now the rest of the string must parse as a valid version string; if it does, it's selected, if it doesn't, it's eliminated.
	
	For example, if the "filename prefix" is `libfoo`, these will match:
	- `libfoo-v1.0.0.tar.gz`
	- `libfoo-2.3-alpha.tar.xz`
	
	However, these will be filtered out:
	- `libasdf-1.0.0.tar.xz` (does not have the prefix)
	- `libfoo3.4.tar.xz` (the name and the version must be separated with a `-`)
	- `libfoo-1.0.0.zip` (not a tar archive)
	- `libfoo-alpha.tar.gz` (does not have a valid version string)
- Glob options additionally accept a leading `!` character to negate the pattern.


### `boost`
Checks for the BOOST C++ library.

It does not have any options.

### `gh`
Checks Github releases for automatic tarballs or release artifacts.

This command directly queries the Github API. It will use the value in the `AB_GITHUB_TOKEN` environment variable if it's set; this way you're less likely to get rate-limited.

The only mandatory argument is the repository string in the form `OWNER/REPO`.

If no additional options are provided, it will print the automatic tarball URL; these are created by Github automatically upon a release.
To get release artifacts, provide one of the below:
- A 2nd positional argument: The filename prefix.
- `-r --regex`: Use a regular expression to select an artifact.
- `-g --glob`: Use a UNIX style glob pattern to select an artifact.

### `gnu`
Check for releases from `ftp.gnu.org`, or another site with the same layout.

The only required argument is the project's name.

- If the name argument starts with `https://` or `http://`, the program will look in that site instead of `ftp.gnu.org`; in that case the project name is inferred as being the last component in the url.
- Otherwise the program will visit `https://ftp.gnu.org/pub/gnu/<name>`.

The second argument is the filename prefix; if it's not provided, the project name will be used.

## `gnupg`
Check for releases from `gnupg.org`.

The only required argument is the project name.

The URL to be visited is determined as `https://www.gnupg.org/ftp/gcrypt/<name>`.

The second argument is the optional filename prefix; if it's not provided, the project name is used.

### `gnome`
Check for releases from `https://download.gnome.org`.

The only required option is the project name.

The URL to visit is determined as `https://download.gnome.org/sources/<name>`.

The second argument is the optional filename prefix; if it's not provided, the project name is used.

### `msys`
Check for MSYS2 binary packages.

The only required argument is the package name.

By default, the packages from the mingw-w64-x86_64 repository will be searched; this is the one that links to the older `msvcrt` runtime.

Options:
- `-m --msvcrt`: Look in the mingw-w64-x86_64 repository; This is the one that links with the older C runtime `msvcrt`.
- `-u --ucrt`: Look in the mingw-w64-ucrt-x86_64 repository; this is the one with the packages that link with the newer Universal C Runtime `ucrt`.

### `openbsd`
Check for releases from `ftp.openbsd.org`.

The only required argument is the project name.

The page to be crawled is determined as `https://ftp.openbsd.org/pub/OpenBSD/<name>`.

The second argument is the optional filename prefix; if it's not provided, the project name is used.

### `sf`
Check releases from `sourceforge.net`.

The only required argument is the project name.

The second argument is the optional filename prefix; if it's not provided, the project name is used.

Due to Sourceforge having rather arbitrary download directory hierarchies, this subcommand is more complicated than others.

Options:
- `-g --glob`: Filter filenames with a glob pattern.
- `-r --regex`: Filter filenames with a regular expression.
- `--dir`: Start the search from a particular directory name.
- `-d --dir-glob`: On the top-level directory, only traverse directories that match a glob pattern.
- `-D --dir-regex`: On the top-level directory, only traverse directories that match a regular expression.

The directory to start from is determined as follows:
- If the `--dir` option is provided: `https://sourceforge.net/projects/{}/files/<name>/<dir>`
- If the `--dir-glob` or the `--dir-regex` options are set, then the directories under the root that match the filters are selected.
- Otherwise, the search starts from the root: `https://sourceforge.net/projects/{}/files/<name>`.
	- As an optimization, if the URL `https://sourceforge.net/projects/{}/files/<name>/<name>` exists, the search starts there.

### `sqlite`
Check for the latest SQLite release.

This subcommand has no options.

Due to the downloads on `https://www.sqlite.org/download.html` being dynamically generated using JavaScript, the only way to check for updates is with a headless browser.
The program will try to instantiate one; if none is found or an error occurs, a fallback hard-coded version is printed.

## `ab-helper dl`
Downloads a file form the internet.

Since the helper isn't meant to face the user, this subcommand is fairly primitive:
- Unlike `wget`, it won't infer file names; the caller has to provide a file to write to.
- It has very few options.

```shell
# Download and save to foo.tar.xz
ab-helper dl -O foo.tar.xz https://example.com/foo.tar.xz
# Write the contents into stdout instead of saving to a file
# This can be piped to, say, bsdtar
ab-helper dl -O - https://example.com/foo.tar.xz
```

## `ab-helper detect-crt`
Detects the dynamically linked C runtime of a PE binary.

The only required argument is the path of a PE executable.

IF it's a valid PE binary that links to the C runtime dynamically, it will print `msvcrt` or `ucrt`.
If the binary does not link with the runtime dynamically, the detection will fail.

## `ab-helper edit-lt`
Edit a Libtool `.la` file.

Required options:
- `-p --path`: Path to the `.la` file.
- `--relocate`: Edit the file so that library and include paths point to a new location.

This subcommand is used while installing an MSYS2 binary package into the autobuild prefix.

## `ab-helper edit-pc`
Edit a pkg-config `.pc` file.

Required options:
- `-p --path`: Path to the `.pc` file.

The edits are specified in a tiny custom syntax:
- To set a variable, use `$var=value` syntax (Make sure to escape the dollar sign so the shell does not interpret it).
- To edit a field that stores a list of values (such as `Libs`, `Requires`, `Cflags.private`):
	
	`<Field><op><values>`
	
	Where `<op>` is one of:
	- `=`: Replace with new values
	- `+=`: Append values to the right
	- `<+=`: Append values to the left
	- `-=`: Remove values
	- `?=`: Set values if the field does not exist or is empty
- And to set a field that's not a list (Version, Description, URL):
	- `<Field>=<value>` to set
	- `<Field>?=<value>` to set if it's not set

This subcommand is used to "monkey patch" faulty `.pc` files as well as to relocate `.pc` files into a different prefix.

## `ab-helper merge-dir`
Merge contents of one directory into another.

Required arguments:
- `<source>`: The directory containing files that will be copied into the other.
- `<dest>`: The directory the files will be copied into.

This subcommand is similar to `rsync` onl ocal directories.
It won't check for file creation/modification times; if files from `source` exist on `dest`, they are overwritten.

This subcommand is used while installing binary packages from MSYS2 repositories.

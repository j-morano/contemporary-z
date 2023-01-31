<p align="center">
  <img src="assets/contemporary-z_header.png" alt="contemporary z"><br>
  A contemporary version of  <tt>z - jump around</tt>.
</p>

<p align="center">
  <a href="#key-features">Key Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#how-to-use">How To Use</a>
</p>


https://user-images.githubusercontent.com/48717183/212571284-a4c27d27-203c-47a8-afd9-6856f1aaff96.mp4


## Key Features

**Contemporary-z** (`cz`) is a modern version of [z - jump around](https://github.com/rupa/z). It is implemented in Rust+SQLite, and utilizes temporary files for the inter-process communication with the shell.

For the time being, `cz` is available for [fish shell](https://github.com/fish-shell/fish-shell), [Bash](https://www.gnu.org/software/bash/) and [Zsh](https://www.zsh.org/). Furthermore, since it is written in Rust and uses an SQLite database, it should be relatively easy to adapt it to more shells; it is only necessary to translate into the language of the new shell the _z scripts_ (e.g. `z.sh`).

## Installation

### Availability

Linux-only.

- fish shell
- Bash
- Zsh

### Install using binary release (recommended)

To install the program using the binary release, copy and paste the following commands in your terminal.

> NOTE: This is only for installing the first time, for updating, see below.

> NOTE: In the following snippets, you can replace `$HOME/.local/bin/` by any other dir in `$PATH`.

#### Fish

```shell
wget https://github.com/j-morano/contemporary-z/releases/latest/download/cz
chmod +x cz
cp cz $HOME/.local/bin/
wget https://raw.githubusercontent.com/j-morano/contemporary-z/main/z.fish
mkdir -p $HOME/.config/fish/functions
cp z.fish $HOME/.config/fish/functions
```

#### Bash/Zsh

```shell
wget https://github.com/j-morano/contemporary-z/releases/latest/download/cz
chmod +x cz
cp cz $HOME/.local/bin/
wget https://raw.githubusercontent.com/j-morano/contemporary-z/main/z.sh
cat z.sh >> $HOME/.bashrc
```

#### Update

```shell
wget https://github.com/j-morano/contemporary-z/releases/latest/download/cz
chmod +x cz
cp cz $HOME/.local/bin/
```

### Install from source

To install `cz` from source, it is required to have installed [Cargo](https://doc.rust-lang.org/cargo/), the Rust _package manager_. You can install Rust, with `rustc` and `cargo`, following the instructions on [this](https://www.rust-lang.org/tools/install) page or from the official repositories of your distribution.

If this requirement is met, it is only necessary to clone the repository and run the specific installation script for the desired shell.


#### Debian-based distros:

In Debian-based distros (like Ubuntu), it is necessary to install the `build-essential` meta-package: 

```shell
sudo apt install build-essential
```

#### Arch Linux

In Arch-based distros, it is necessary to install the `base-devel` meta-package:
```
sudo pacman -S base-devel
```

#### Repository cloning

Using SSH:
```shell
git clone git@github.com:sonarom/contemporary-z.git
```

Using HTTPS:
```shell
git clone https://github.com/sonarom/contemporary-z.git
```

#### Install using Cargo

```shell
$HOME/.cargo/bin/cargo install --path cz
```

Then, depending on the shell, do the following.

> NOTE: ensure that `$HOME/.cargo/bin/` is in `$PATH`.

> NOTE: You must be inside the repository folder (`contemporary-z`) to run the commands as shown below.

#### Fish

```shell
mkdir -p $HOME/.config/fish/functions
cp z.fish $HOME/.config/fish/functions
```

#### Bash/Zsh

```shell
cat z.sh >> $HOME/.bashrc
```

## How to use

The default alias of Contemporary-z is `z`. However, if a different alias is preferred, it can be easily changed in the installation scripts. Hereafter, `cz` refers to the entire application, and `z` refers to the command.


### Usage:

```
Usage: z [OPTION]... [DIRECTORY|SUBSTRING]...

FUNCTIONALITY:
  * If no option nor directory or substrings are specified, 'cz' prints a
    numbered list of the most 'frecent' directories to select one of them by
    introducing its number.
  * If a directory alias is introduced, 'cz' does 'cd' to the directory.
  * If a directory name is introduced, 'cz' jumps to the directory (if
    available) and adds it to the directories database (if it is not already
    added).
  * If a substring or substrings are introduced, 'cz' searches in the database
    for coincidences. Then, if 'substring' is 'shortest' or the number of
    matches is equal to 1, it 'cd's to the directory with the shortest pathname.
    If 'substring' is equal to 'score', it goes to the directory with the
    highest score. Otherwise, it prints the interactive selection menu. If there
    is only one result, it always 'cd's to it.

OPTIONS:
Mandatory arguments to long options are mandatory for short options too.
  -                          go to the previous directory.
  =                          go to the current directory.
  -a [ALIAS] DIRECTORY       add directory alias; if only the directory is
                               introduced, its alias is removed; if only an
                               alias is introduced, the programs shows the
                               directory list to select one of them.
      --clear                clear the directories database.
  -f SUBSTRING               force substring match list for SUBSTRING
  -i                         interactive selection (using a numbered list) of
                               the subdirectories of the current directory.
      --ih                   interactive selection, but including hidden
                               directories.
      --id                   interactive selection, for directories only
                               (ignore congifuration option 'show_files').
  -l [NUMBER]                list a certain NUMBER of directories by 'frecency';
                               if no NUMBER is provided, the max_results number
                               from configuration is used.
      --list-all             list all the directories of the database
  -r                         remove a directory from the database, interactively.
      --remove-alias         remove an alias, interactively.
      --sync                 sync directories (remove all non-existent
                                directories).
      --help     display this help and exit.
  -v, --version              display version information and exit.

Exit status:
 0  if OK,
 1  if minor problems (e.g., cannot access subdirectory)

Full documentation <https://github.com/sonarom/contemporary-z>
```


### Configuration

`cz` supports some configuration options. These options must be set in TOML format in a file with the following path: `~/.config/contemporary-z/cz.toml`.

#### Options:

* `theme`: `string`. Color theme.
    + Allowed values: `dark`, `bright`
* `abs_paths`: `bool`. Record directories using full paths or relative paths. With the latter option, shown directories will vary from one directory to another.
* `compact_paths`: `bool`. Replace `/home/<username>` by `~` and `/run/media/<username>` by `>`.
* `max_results`: `int`. Maximum results to show in the directory list.
* `database_path`: `string`. Directory where the directories database is/will be located.
* `substring`: `string`. Which dir to select when substring(s) are introduced.
    + Allowed values:
        - 'shortest': go to the directory with the shortest path name.
        - 'score': go to the directory with the highest score (most 'frecent' dir).
        - 'none': show selection list.
* `show_files`: `string`. Whether to show non-dir files, and where, in interactive selection.
    + Allowed values:
        - 'top': show files on top of dirs.
        - 'bottom': show files under the dirs.
        - 'none': do not show files.
* `nav_start_number`: `int`. Start number for interactive navigation, that is, the number that the parent directory will have.
   + Recommended values: 1 or 0.


#### Default config

```toml
# ~/.config/contemporary-z/cz.toml

theme = 'dark'
max_results = 9
abs_paths = true
compact_paths = true
database_path = '$HOME/.local/share/contemporary-z/directories.db'
substring = 'shortest'
show_files = 'none'
nav_start_number = 1
```

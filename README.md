<p align="center">
  <img src="assets/contemporary-z_header.png" alt="contemporary z"><br>
  A contemporary version of  <tt>z - jump around</tt>.
</p>

<p align="center">
  <a href="#key-features">Key Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#how-to-use">How To Use</a>
</p>


## Key Features

**Contemporary-z** (`cz`) is a modern version of [z - jump around](https://github.com/rupa/z). It is implemented in Rust+SQLite, and utilizes temporary files for the inter-process communication with the shell.

`cz` is at an early stage of development, so it lacks many of the functionalities available in the original `z`. However, its design makes it easily extensible, so these functionalities will not take long to be incorporated.

For the time being, `cz` is available for [fish shell](https://github.com/fish-shell/fish-shell), [Bash](https://www.gnu.org/software/bash/) and [Zsh](https://www.zsh.org/). Furthermore, since it is written in Rust and uses an SQLite database, it should be relatively easy to adapt it to more shells; it is only necessary to translate into the language of the new shell the installation and runtime scripts.

## Installation

### Availability

- fish shell
- Bash
- Zsh

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

#### Run the installation script

> You must be inside the repository folder (`contemporary-z`) to run them as shown below.

##### Fish

```fish
./bin/install.fish
```

##### Bash

```bash
./bin/install.sh
```

##### Zsh

```zsh
./bin/install.zsh
```


#### Uninstallation

To uninstall `contemporary-z`, you can use the uninstallation scripts under `bin/` the same way as installation ones:

```sh
./bin/uninstall.[fish,sh,zsh]
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
    for coincidences. Then, if 'substring_shortest' is 'true' ir the number of
    matches is equal to 1, it 'cd's to the directory with the shortest pathname.
    Else, if there are more than 1 match and 'substring_shortest' is 'false',
    'cz' prints the interative selection menu.

OPTIONS:
Mandatory arguments to long options are mandatory for short options too.
  -                          go to the previous directory
  =                          go to the current directory
  -a [ALIAS] DIRECTORY       add directory alias; if only the directory is
                               introduced, its alias is removed
      --clear                clear the directories database
  -f SUBSTRING               force substring match list for SUBSTRING
  -i                         interactive selection (using a numbered list) of
                               the subdirectories of the current directory.
  -ih                        interactive selection, but including hidden
                               directories
  -l [NUMBER]                list a certain NUMBER of directories by 'frecency';
                               if no NUMBER is provided, the max_results number
                               from configuration is used.
      --list-all             list all the directories of the database
  -r                         remove directories from the database, by
                               introducing, interactively, its numbers separated
                               by spaces.
      --sync                 sync directories (remove all non-existent
                                directories)
      --help     display this help and exit
  -v, --version              display version information and exit

Exit status:
 0  if OK,
 1  if minor problems (e.g., cannot access subdirectory)

Full documentation <https://github.com/sonarom/contemporary-z>
```


### Configuration

`cz` supports some configuration options. These options must be set in TOML format in a file with the following path: `~/.config/cz.toml`.

#### Options:

* `theme`: `string`. Color theme.
    + Allowed values: `dark`, `bright`
* `abs_paths`: `bool`. Record directories using full paths or relative paths. With the latter option, shown directories will vary from one directory to another.
* `compact_paths`: `bool`. Replace `/home/<username>` by `~` and `/run/media/<username>` by `>`.
* `max_results`: `int`. Maximum results to show in the directory list.
* `database_path`: `string`. Directory where the directories database is/will be located.
* `substring_shortest`: `bool`. Directly access the dir with the shortest pathname that matches the substring(s).

#### Default config

```toml
# ~/.config/cz.toml

theme = 'dark'
max_results = 9
abs_paths = true
compact_paths = true
database_path = '$HOME/.local/share/cz/'
substring_shortest = true
```


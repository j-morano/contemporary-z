<p align="center">
  <img src="doc/contemporary-z_header.png" alt="contemporary z"><br>
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

In Debian-based distros, it is necessary to install the developer package of the `sqlite3` library:
```shell
sudo apt install libsqlite3-dev
```

##### Ubuntu

To install `cz` in Ubuntu, it is necessary to install the developer package of the sqlite3 library:

```
sudo apt install libsqlite3-dev
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

```fish
z [options] [directory or substrings]
```

1. If no option nor directory or substrings are specified, `cz` prints a numbered list of the most 'frecent' directories to select one of them by introducing its number.
<!--- the most 'frecent' directories -->

2. If a directory alias is introduced, `cz` does `cd` to the directory.

3. If a directory name is introduced, `cz` jumps to the directory (if available) and adds it to the directories database (if it is not already added).

4. If a substring or substrings are introduced, `cz` searches in the database for coincidences, and does `cd` to the uppermost directory.

<!--- If there is only one coincidence, `cz` accesses the directory directly. If there are 2 or more coincidences, `cz` outputs the list, as in the case 1. -->


### Options:

* `--clear`: clear the directories database.
* `-`: go to the previous directory.
* `=`: go to the current directory.
* `-b`: execute shell command in background.
* `-l [number]`: list a certain `number` of directories by 'frecency'; if no `number` is provided, the `max_results` number is used.
* `-i`: interactive selection (using a numbered list) of the subdirectories of the current directory.
* `-r`: remove directory. Works the same as regular `cz` but for removing.
* `-a`: add directory alias. If only the directory is introduced, its alias is removed.


### Configuration

`cz` supports some configuration options. These options must be set in TOML format in a file with the following path: `~/.config/cz.toml`.

#### Options:

* `theme`: `dark|bright`. Color theme.
* `abs_paths`: `true|false`. Record directories using full paths or relative paths. With the latter option, shown directories will vary from one directory to another.
* `compact_paths`: `true|false`. Replace `/home/<username>` by `~` and `/run/media/<username>` by `>`.
* `max_results`: any number. Maximum results to show in the directory list.
* `database_path`: any string. Directory where the directories database is/will be located.

#### Default config

```toml
# ~/.config/cz.toml

theme = 'dark'
max_results = 9
abs_paths = true
compact_paths = true
database_path = '$HOME/.local/share/cz/'
```


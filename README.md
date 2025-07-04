<p align="center">
  <img src="assets/contemporary-z_header_dark.png" alt="contemporary z"><br>
  A contemporary version of  <tt>z - jump around</tt>.
</p>

<p align="center">
  <a href="#key-features">Key Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#how-to-use">How To Use</a>
</p>


https://user-images.githubusercontent.com/48717183/212571284-a4c27d27-203c-47a8-afd9-6856f1aaff96.mp4


## Key Features

**Contemporary-z** (`cz`) is a modern version of [z - jump around](https://github.com/rupa/z). It is implemented in Rust and utilizes temporary files for the inter-process communication with the shell.

For the time being, `cz` is available for [fish shell](https://github.com/fish-shell/fish-shell), [Bash](https://www.gnu.org/software/bash/) and [Zsh](https://www.zsh.org/). Furthermore, it should be relatively easy to adapt it to more shells; it is only necessary to translate into the language of the new shell the _z scripts_ (e.g. `z.sh`).

## Installation

### Availability

Linux-only.

- fish shell
- Bash
- Zsh

### Install/update using installation script (recommended)

To install `cz`, you can just run the installation script.

```shell
python -c "$(curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/j-morano/contemporary-z/main/install)"
```


### Install manually using binary release

To install the program using the binary release, copy and paste the following commands in your terminal.

> [!NOTE]
> This is only for installing the first time, for updating, see [Update](#update).

> [!TIP]
> In the following snippets, you can replace `$HOME/.local/bin/` by any other dir in `$PATH`.


#### Fish

```shell
wget https://github.com/j-morano/contemporary-z/releases/latest/download/cz
chmod +x cz
mv cz $HOME/.local/bin/
wget https://raw.githubusercontent.com/j-morano/contemporary-z/main/z.fish
mkdir -p $HOME/.config/fish/functions
mv z.fish $HOME/.config/fish/functions
```

#### Bash/Zsh

```shell
wget https://github.com/j-morano/contemporary-z/releases/latest/download/cz
chmod +x cz
mv cz $HOME/.local/bin/
wget https://raw.githubusercontent.com/j-morano/contemporary-z/main/z.sh
cat z.sh >> $HOME/.bashrc
```

#### Update

```shell
wget https://github.com/j-morano/contemporary-z/releases/latest/download/cz
chmod +x cz
mv cz $HOME/.local/bin/
```

### Install manually from source

To install `cz` from source, it is required to have installed [Cargo](https://doc.rust-lang.org/cargo/), the Rust _package manager_. You can install Rust, with `rustc` and `cargo`, following the instructions on [this](https://www.rust-lang.org/tools/install) page or from the official repositories of your distribution.

If this requirement is met, it is only necessary to clone the repository and run the specific installation script for the desired shell.


#### Clone the repository

```shell
git clone https://github.com/j-morano/contemporary-z.git
```

#### Install using Cargo

```shell
$HOME/.cargo/bin/cargo install --path .
```

> [!IMPORTANT]
> Ensure that `$HOME/.cargo/bin/` is in `$PATH` after the installation.

#### Add the function to the shell

> [!IMPORTANT]
> You must be inside the repository folder (`contemporary-z`) to run the commands as shown below.

##### Fish

Add the function to fish functions.

```shell
mkdir -p $HOME/.config/fish/functions
mv z.fish $HOME/.config/fish/functions
```

##### Bash/Zsh

Add the code from `z.sh` to `.bashrc`. For example:

```shell
cat z.sh >> $HOME/.bashrc
```

## How to use

The default alias of Contemporary-z is `z`. However, if a different alias is preferred, it can be easily changed in the installation scripts. Hereafter, `cz` refers to the entire application, and `z` refers to the command.


### Usage

To see the usage of `cz`, you can just run it with the `--help` argument.

```
$ z --help
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
    for coincidences. The selected directory will depend on the 'substring'
    configuration option (see Configuration section below). If there is only
    one result, it always 'cd's to it, regardless of the option.

OPTIONS:
Mandatory arguments to long options are mandatory for short options too.
  -                          go to the previous directory.
  =                          go to the current directory.
  -a [ALIAS] DIRECTORY       add directory alias; if only the directory is
                               introduced, its alias is removed; if only an
                               alias is introduced, the programs shows the
                               directory list to select one of them.
      --clear                clear the directories database.
  -b SUBSTRING               force substring matching by basename.
  --database-path            show the path of the directories database.
  -e SUBSTRING               force substring matching by score.
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
  -t SUBSTRING               force substring matching by shortest path.
      --sync                 sync directories (remove all non-existent
                                directories).
      --help     display this help and exit.
  -v, --version              display version information and exit.

Exit status:
 0  if OK,
 1  if minor problems (e.g., cannot access subdirectory)


CONFIGURATION:
'cz' supports some configuration options. These options must be set in a file
with the following path: '~/.config/contemporary-z/cz.conf' using the format
'option = value'.

Configuration options:
  theme: string. Color theme.
      * Allowed values: 'dark', 'bright'
  abs_paths: bool. Record directories using full paths or relative paths.
      With the latter option, shown directories will vary from one directory to
      another.
  compact_paths: bool. Replace '/home/<username>' by '~' and
      '/run/media/<username>' by '>'.
  max_results: int. Maximum results to show in the directory list.
  database_path: string. Directory where the directories database is/will
      be located.
  substring: string. Which dir to select when substring(s) are introduced.
      * Allowed values:
          - 'shortest': go to the directory with the shortest path name.
          - 'score': go to the directory with the highest score (most 'frecent'
              dir).
          - 'none': show selection list.
          - 'basename': go to the directory with the highest score (most 'frecent'
              dir) whose _basename_ matches the substring(s).
  show_files: string. Whether to show non-dir files, and where, in
      interactive selection.
      * Allowed values:
          - 'top': show files on top of dirs.
          - 'bottom': show files under the dirs.
          - 'none': do not show files.
  nav_start_number: int. Start number for interactive navigation, that is,
      the number that the parent directory will have.
      * Recommended values: 1 or 0.


Default config:
-------------------------------------------------------------------
# ~/.config/contemporary-z/cz.conf

theme = dark
max_results = 9
abs_paths = true
compact_paths = true
database_path = $HOME/.local/share/contemporary-z/directories.dir
substring = score
show_files = none
nav_start_number = 1
-------------------------------------------------------------------

Source code: <https://github.com/j-morano/contemporary-z>
```

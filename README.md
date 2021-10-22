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

**Contemporary-z** (`cz`) is a modern and improved version of [z - jump around](https://github.com/rupa/z). It is implemented in Rust+SQLite, and utilizes temporary files for the inter-process communication with the shell.

`cz` is at an early stage of development, so it lacks many of the functionalities available in the original `z`. However, its design makes it easily extensible, so these functionalities will not take long to be incorporated.

For the time being, `cz` is available for [fish shell](https://github.com/fish-shell/fish-shell), [Bash](https://www.gnu.org/software/bash/) and [Zsh](https://www.zsh.org/). Furthermore, since it is written in Rust and uses an SQLite database, it should be relatively easy to adapt it to more shells; it is only necessary to translate into the language of the new shell the installation and runtime scripts.


## Installation

### Availability

- fish shell
- Bash
- Zsh

### Install from source

To install `cz` from source, it is required to have installed [Cargo](https://doc.rust-lang.org/cargo/), the Rust _package manager_.

If this requirement is met, it is only necessary to clone the repository and run the specific installation script for the desired shell.

#### Repository cloning

Using SSH:
```shell
git clone git@github.com:sonarom/contemporary-z.git
```

Using HTTPS:
```shell
git clone https://github.com/sonarom/contemporary-z.git
```

#### Fish

```fish
./install.fish
```

#### Bash

```bash
./install.sh
```

#### Zsh

```zsh
./install.zsh
```


## How to use


### Usage:

```fish
z [options] [folder or substrings]
```

1. If no option nor folder or substrings are specified, `z` prints a numbered list of the most frequent directories to select one of them by introducing its number.

2. If a folder name is introduced, `z` jumps to the folder (if available) and adds it to the folders database (if it is not already added).

3. If a substring or substrings are introduced, `z` searches in the database for coincidences. If there is only one coincidence, `z` accesses the folder directly. If there are 2 or more coincidences, `z` outputs the list, as in the case 1.



### Options:

* `--clear`: clears the folders database.



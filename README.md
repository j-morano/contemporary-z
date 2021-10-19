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

**Contemporary-z** is a modern and improved version of [z - jump around](https://github.com/rupa/z). It is implemented in Rust+SQLite, and utilizes temporary files for the interprocess communication with the shell.

Contemporary-z is in early development, so it lacks many of the functionalities available in the original `z`. However, its design makes it easily extensible, which makes it possible not to delay the incorporation of these functionalities for too long.

For the time being, Contemporary-z already supports [fish shell](https://github.com/fish-shell/fish-shell) and [bash](https://www.gnu.org/software/bash/). Furthermore, since it is written in Rust and uses an SQLite database, it should be relatively easy to adapt it to more shells; it is only necessary to incorporate new installation and runtime scripts for the new shell.


## Installation

At the moment, Comeplementary-z supports both fish shell and bash. To install it from source, it is only necessary to run the specific installation script for the desired shell.

### Fish

```fish
./install.fish
```

### Bash

```bash
./install.sh
```


## How to use


### Usage:

```fish
z [options] [folder]
```

If no option nor folder is specified, `z` prints a numbered list of the most frequent directories to select one of them by introducing the number of the desired folder.

If a folder name is itroduced, `z` jumps to the folder, if available, and adds it to the folders database (if it is not already added).



### Options:

* `--clear`: clears the folders database.




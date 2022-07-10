#!/usr/bin/env bash

sed -n -i "/alias z\s*=\s*.*/!p" $HOME/.bashrc
$HOME/.cargo/bin/cargo uninstall cz
rm -r $HOME/.local/share/cz/

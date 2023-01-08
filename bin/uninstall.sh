#!/bin/sh

sed -n -i "/alias z\s*=\s*.*/!p" $HOME/.bashrc
$HOME/.cargo/bin/cargo uninstall cz
rm -r $HOME/.local/share/contemporary-z/

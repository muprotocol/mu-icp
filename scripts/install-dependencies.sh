#!/usr/bin/env bash

wget https://github.com/dfinity/pocketic/releases/download/4.0.0/pocket-ic-x86_64-linux.gz &&
mv pocket-ic-x86_64-linux.gz pocket-ic.gz &&
gzip -d pocket-ic.gz &&
mkdir -p ~/.local/share/dfx/bin/ &&
mv pocket-ic ~/.local/share/dfx/bin/ && 
chmod +x ~/.local/share/dfx/bin/pocket-ic

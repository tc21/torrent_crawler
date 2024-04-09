#! /bin/bash

cargo build --release --all-features

if [[ -d out ]]; then
    rm -r out/
fi;

mkdir out/

cp target/release/add_to_transmission out/
cp target/release/torrent_crawler out/
cp config.json out/

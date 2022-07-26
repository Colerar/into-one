#!/usr/bin/env zsh

# build script for mac
# brew install SergioBenitez/osxct/x86_64-unknown-linux-gnu

cargo build --release

TARGET_CC=x86_64-unknown-linux-gnu cargo build --release --target x86_64-unknown-linux-gnu

cargo build --release --target x86_64-pc-windows-gnu

cd target

rm ./dists/*.7z >/dev/null
rm ./dists/*.gz >/dev/null

echo "Creating dists..."

7zmt () {
  7z -mmt16 -mx9 "$@" 1>/dev/null &
}

mkdir dists &>/dev/null

targzflat () {
  tar --xform 's:^.*/::' -czf "$@" 1>/dev/null &
}

GZIP=-9

targzflat ./dists/into-one-mac-x86_64.tar.gz ./release/into-one

targzflat ./dists/into-one-linux-x86_64.tar.gz ./x86_64-unknown-linux-gnu/release/into-one

targzflat ./dists/into-one-win-x86_64.tar.gz ./x86_64-pc-windows-gnu/release/into-one.exe

targzflat ./dists/into-one-mac-x86_64-bundle.tar.gz ./release/into-one ./ffmpeg/mac/ffmpeg

targzflat ./dists/into-one-linux-x86_64-bundle.tar.gz ./x86_64-unknown-linux-gnu/release/into-one ./ffmpeg/linux/ffmpeg

7zmt a ./dists/into-one-win-x86_64-bundle.7z ./x86_64-pc-windows-gnu/release/into-one.exe ./ffmpeg/win/ffmpeg.exe

wait

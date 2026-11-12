#!/bin/sh
set -eu
cd "$(dirname "$0")"
swift build -c release --disable-sandbox
bin_dir=$(swift build -c release --disable-sandbox --show-bin-path)
app=build/multi-link.app
mkdir -p "$app/Contents/MacOS"
cp "$bin_dir/multi-link-menubar" "$app/Contents/MacOS/multi-link-menubar"
cp Info.plist "$app/Contents/Info.plist"
codesign --force --sign - -r='designated => identifier "dev.multi-link.menubar"' "$app"
codesign --verify --strict "$app"
printf '%s\n' 'Built and verified build/multi-link.app'

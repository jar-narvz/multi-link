#!/bin/sh
set -eu
cd "$(dirname "$0")"
app=build/multi-link.app
rm -rf "$app"
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
(cd ../.. && cargo build --release --locked -p multi-link)
swift build -c release --disable-sandbox
bin_dir=$(swift build -c release --disable-sandbox --show-bin-path)
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$bin_dir/multi-link-menubar" "$app/Contents/MacOS/multi-link-menubar"
cp ../../target/release/multi-link "$app/Contents/MacOS/multi-link"
cp Info.plist "$app/Contents/Info.plist"
iconutil -c icns Resources/AppIcon.iconset -o "$app/Contents/Resources/AppIcon.icns"
icon_name=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIconFile' "$app/Contents/Info.plist")
if [ ! -f "$app/Contents/Resources/$icon_name.icns" ]; then
    printf '%s\n' "Missing bundle icon: $icon_name.icns" >&2
    exit 1
fi
codesign --force --sign - -i dev.multi-link.cli -r='designated => identifier "dev.multi-link.cli"' "$app/Contents/MacOS/multi-link"
codesign --force --sign - -r='designated => identifier "dev.multi-link.menubar"' "$app"
codesign --verify --strict "$app/Contents/MacOS/multi-link"
codesign --verify --strict --deep "$app"
printf '%s\n' 'Built and verified build/multi-link.app (bundled CLI, app, icon)'

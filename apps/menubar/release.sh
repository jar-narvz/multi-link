#!/bin/sh
set -eu
cd "$(dirname "$0")"

if [ "$(uname -m)" != arm64 ]; then
    printf '%s\n' 'The arm64 release must be built on an arm64 host.' >&2
    exit 1
fi

./build.sh
app=build/multi-link.app
version=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$app/Contents/Info.plist")
case "$version" in
    ''|*[!0-9A-Za-z._-]*)
        printf '%s\n' "Invalid bundle version: $version" >&2
        exit 1
        ;;
esac
dmg="$(pwd)/build/multi-link-$version-arm64.dmg"
work_dir=$(mktemp -d "${TMPDIR:-/tmp}/multi-link-release.XXXXXX")
mount_dir="$work_dir/mount"
mounted=0

cleanup() {
    status=$?
    trap - 0 HUP INT TERM
    if [ "$mounted" -eq 1 ]; then
        if ! hdiutil detach "$mount_dir"; then
            printf '%s\n' "Could not detach $mount_dir; keeping $work_dir for cleanup." >&2
            exit 1
        fi
    fi
    rm -rf "$work_dir"
    exit "$status"
}
trap cleanup 0
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM

mkdir "$work_dir/stage" "$mount_dir"
ditto "$app" "$work_dir/stage/multi-link.app"
ln -s /Applications "$work_dir/stage/Applications"
rm -f "$dmg"
hdiutil create -srcfolder "$work_dir/stage" -format UDZO -volname "multi-link $version" "$dmg"
hdiutil verify "$dmg"
# Set this before attach so the trap also tries to detach after a partial mount.
mounted=1
hdiutil attach -readonly -nobrowse -noautoopen -mountpoint "$mount_dir" "$dmg"
mounted_app="$mount_dir/multi-link.app"
codesign --verify --strict "$mounted_app/Contents/MacOS/multi-link"
codesign --verify --strict --deep "$mounted_app"
icon_name=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIconFile' "$mounted_app/Contents/Info.plist")
case "$icon_name" in
    *.icns) ;;
    *) icon_name="$icon_name.icns" ;;
esac
if [ ! -f "$mounted_app/Contents/Resources/$icon_name" ]; then
    printf '%s\n' "Missing mounted bundle icon: $icon_name" >&2
    exit 1
fi
if [ ! -L "$mount_dir/Applications" ] || [ "$(readlink "$mount_dir/Applications")" != /Applications ]; then
    printf '%s\n' 'The mounted Applications link must point to /Applications.' >&2
    exit 1
fi
hdiutil detach "$mount_dir"
mounted=0
printf '%s\n' "$dmg"

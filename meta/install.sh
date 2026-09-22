#!/usr/bin/env bash
set -eu

app_id="com.nithinrdy.audiopass"
binary_name="audiopass"
data_dir="${XDG_DATA_HOME:-$HOME/.local/share}"

relative_script_dir="$(dirname -- "${BASH_SOURCE[0]}")"
absolute_script_dir="$(cd -- "$relative_script_dir" && pwd)"

binary_destination="$HOME/.local/bin/$binary_name"
install -Dm0755 "$absolute_script_dir/$binary_name" "$binary_destination"
install -Dm0644 "$absolute_script_dir/$app_id.metainfo.xml" "$data_dir/metainfo/$app_id.metainfo.xml"

desktop_destination="$data_dir/applications/$app_id.desktop"
install -Dm0644 "$absolute_script_dir/$app_id.desktop.template" "$desktop_destination"
printf '\nExec="%s"' "$binary_destination" >> "$desktop_destination"

install -Dm0644 "$absolute_script_dir/icon.svg" "$data_dir/icons/hicolor/scalable/apps/$app_id.svg"
install -Dm0644 "$absolute_script_dir/icon-32.png" "$data_dir/icons/hicolor/32x32/apps/$app_id.png"
install -Dm0644 "$absolute_script_dir/icon-48.png" "$data_dir/icons/hicolor/48x48/apps/$app_id.png"
install -Dm0644 "$absolute_script_dir/icon-64.png" "$data_dir/icons/hicolor/64x64/apps/$app_id.png"
install -Dm0644 "$absolute_script_dir/icon-128.png" "$data_dir/icons/hicolor/128x128/apps/$app_id.png"
install -Dm0644 "$absolute_script_dir/icon-256.png" "$data_dir/icons/hicolor/256x256/apps/$app_id.png"

printf 'Installed AudioPass to %s\n' "$binary_destination"

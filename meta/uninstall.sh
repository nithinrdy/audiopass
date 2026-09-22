#!/usr/bin/env bash
set -eu

app_id="com.nithinrdy.audiopass"
binary_name="audiopass"
data_dir="${XDG_DATA_HOME:-$HOME/.local/share}"

rm -f "$HOME/.local/bin/$binary_name"
rm -f "$data_dir/metainfo/$app_id.metainfo.xml"
rm -f "$data_dir/applications/$app_id.desktop"

rm -f "$data_dir/icons/hicolor/scalable/apps/$app_id.svg"
rm -f "$data_dir/icons/hicolor/32x32/apps/$app_id.png"
rm -f "$data_dir/icons/hicolor/48x48/apps/$app_id.png"
rm -f "$data_dir/icons/hicolor/64x64/apps/$app_id.png"
rm -f "$data_dir/icons/hicolor/128x128/apps/$app_id.png"
rm -f "$data_dir/icons/hicolor/256x256/apps/$app_id.png"

printf 'AudioPass uninstalled.\n'

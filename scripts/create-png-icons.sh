#!/usr/bin/env bash
set -eu

relative_script_dir="$(dirname -- "${BASH_SOURCE[0]}")"
absolute_script_dir="$(cd -- "$relative_script_dir" && pwd)"
root_dir="$(dirname "$absolute_script_dir")"

input="$root_dir/meta/icon.svg"
output_dir="$root_dir/meta"

# requires imagemagick 7+
for size in 32 48 64 128 256; do
    magick -background none "$input" -resize "${size}x${size}" "$output_dir/icon-${size}.png"
done
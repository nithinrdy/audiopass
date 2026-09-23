#!/usr/bin/env bash
set -eu

relative_script_dir="$(dirname -- "${BASH_SOURCE[0]}")"
absolute_script_dir="$(cd -- "$relative_script_dir" && pwd)"
root_dir="$(dirname "$absolute_script_dir")"

cd "$root_dir"

cargo about generate --locked --output-file THIRD_PARTY about.hbs

printf "Downloading and appending phosphor icons license:\n"
phosphor_icons_license="$(curl --fail "https://raw.githubusercontent.com/phosphor-icons/homepage/master/LICENSE")"
printf '\n------------------------------------------------------------------------------------\n' >> THIRD_PARTY
printf '\n MIT License used by Phosphor Icons \n\n %s\n' "$phosphor_icons_license" >> THIRD_PARTY

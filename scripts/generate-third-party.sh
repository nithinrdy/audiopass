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

# font assets not covered by cargo about https://github.com/emilk/egui#credits Default fonts

printf '\n\n------------------------------------------------------------------------------------\n' >> THIRD_PARTY
printf '\nBundled font notices' >> THIRD_PARTY



# 1. emoji-icon-font https://github.com/jslegers/emoji-icon-font#author
printf '\n\n------------------------------------------------------------------------------------\n' >> THIRD_PARTY
printf 'Emoji Icon Font

Copyright (c) 2014 John Slegers

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.' >> THIRD_PARTY

# 2. hack-regular https://github.com/source-foundry/Hack/blob/master/LICENSE.md
printf '\n\n------------------------------------------------------------------------------------\n' >> THIRD_PARTY
printf "Downloading and appending Hack Regular font license:\n"
hack_regular_license="$(curl --fail "https://raw.githubusercontent.com/source-foundry/Hack/refs/heads/master/LICENSE.md")"
printf '\n Hack Regular \n\n %s\n' "$hack_regular_license" >> THIRD_PARTY

# 3.noto-emoji https://github.com/google/fonts/blob/main/ofl/notoemoji/OFL.txt
printf '\n\n------------------------------------------------------------------------------------\n' >> THIRD_PARTY
printf "Downloading and appending Noto Emoji font license:\n"
noto_emoji_license="$(curl --fail "https://raw.githubusercontent.com/google/fonts/refs/heads/main/ofl/notoemoji/OFL.txt")"
printf '\n Noto Emoji \n\n %s\n' "$noto_emoji_license" >> THIRD_PARTY

# 4. ubuntu-light https://canonical.com/legal/font-licence#disclaimer
printf '\n\n------------------------------------------------------------------------------------\n' >> THIRD_PARTY
printf "Downloading and appending Ubuntu Light license:\n"
ubuntu_light_license="$(curl --fail "https://assets.ubuntu.com/v1/81e5605d-ubuntu-font-licence-1.0.txt")"
printf '\n Ubuntu Light \n\n %s\n' "$ubuntu_light_license" >> THIRD_PARTY

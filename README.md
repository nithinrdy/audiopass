# AudioPass

AudioPass is a Linux desktop app that plays local audio through a virtual microphone. AudioPass interfaces with the system's PipeWire service to create a virtual microphone and route audio through it. The following audio sources are supported:

- any connected physical microphone (equivalent to not using AudioPass),
- any locally running application playing audio
- _[work-in-progress]_ any local audio files

## Minimum Requirements

AudioPass has the following minimum requirements:

- PipeWire `v0.3.49` or above (that's most modern distros)
- glibc `v2.39` or above (run `ldd --version` in your terminal to see your machine's glibc version)

For example, Ubuntu 24.04 LTS, SteamOS 3.6 (as far as I could tell, [SteamOS 3.5 shipped with glibc `v2.37`](https://steamdeck-packages.steamos.cloud/archlinux-mirror/holo-3.5/os/x86_64/) and [SteamOS 3.6 was the first to ship with glibc `v2.39`](https://steamdeck-packages.steamos.cloud/archlinux-mirror/holo-3.6/os/x86_64/), but I didn't have the opportunity to test the app directly on SteamOS), an up-to-date rolling Arch distro, etc. would be adequate.

I'm looking into lowering the minimum glibc requirement to allow the app to run on older distros (like Ubuntu 22.04), but it's not a priority for me right now.

In terms of system resource consumption, AudioPass is ultra lightweight and barely consumes any memory or CPU (CPU use may increase once I add local-audio-file-playback, because it would include a track-progress bar that'd require constant UI repaints as the track progresses, but none of that is in the app at the moment).

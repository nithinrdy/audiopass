# AudioPass

AudioPass is a Linux desktop app that plays local audio through a virtual microphone. It interfaces with the system's PipeWire service to create a virtual microphone and route audio through it. The following audio sources are supported:

- any connected physical microphone (equivalent to not using AudioPass),
- any locally running application playing audio
- _[work-in-progress]_ any local audio files

<img width="2220" height="1756" alt="audiopass-screenshot" src="https://github.com/user-attachments/assets/dc6ea65f-e8cf-40fa-ad57-8164cd32f74c" />

## How does it work?

AudioPass relies on the system's PipeWire service to first set up a virtual mic, then create a virtual capture stream against the user's desired audio source (so let's say their spotify client app, or an offline music player playing songs, stuff like that), and then starts routing audio from the captured stream to the virtual mic.

The cool part (in my opinion at least) is that audio routing is handled not by creating a few `pw-link`s; audio samples are physically carried from the stream to the virtual mic -- the app holds an SPSC ring which acts as a buffer for the samples produced by the user-selected audio source, which get consumed on the other end by the virtual mic. Since both production and consumption of the samples are handled inside the `process()` callbacks of the two pipewire streams, they naturally remain in sync with low risk of underruns and stuff.

Directly interacting with the audio samples also lets me apply a custom gain value directly to them by simply multiplying each sample with a gain multiplier (not to say `pw-link`s don't let you manipulate volume, but processing the underlying samples directly allows a few cool tricks like this, and for example creating an FFT audio visualization by reading the sample values, which I'm considering because it looks cool).

## Download

Check out the [Releases](https://github.com/nithinrdy/audiopass/releases) tab and download the latest archive. A short README is included, contains everything you need to get started. The Help screen in the app explains all the core modes. Except for the external links to the homepage and to this repository, AudioPass is fully offline-only.

## Minimum Requirements

AudioPass has the following minimum requirements:

- PipeWire `v0.3.49` or above (that's most modern distros)
- glibc `v2.39` or above (run `ldd --version` in your terminal to see your machine's glibc version)

For example, Ubuntu 24.04 LTS, SteamOS 3.6 (as far as I could tell, [SteamOS 3.5 shipped with glibc `v2.37`](https://steamdeck-packages.steamos.cloud/archlinux-mirror/holo-3.5/os/x86_64/) and [SteamOS 3.6 was the first to ship with glibc `v2.39`](https://steamdeck-packages.steamos.cloud/archlinux-mirror/holo-3.6/os/x86_64/), but I didn't have the opportunity to test the app directly on SteamOS), an up-to-date rolling Arch distro, etc. would be adequate.

I'm looking into lowering the minimum glibc requirement to allow the app to run on older distros (like Ubuntu 22.04), but it's not a priority for me right now.

In terms of system resource consumption, AudioPass is ultra lightweight and barely consumes any memory or CPU (CPU use may increase once I add local-audio-file-playback, because it would include a track-progress bar that'd require constant UI repaints as the track progresses, but none of that is in the app at the moment).

## Why build this?

I play a lot of Team Fortress 2 and I like playing songs into the lobby through the mic. On Windows I did this with Soundpad. A few months ago I built a new PC and decided to daily-drive Linux. Soundpad doesn't work on Linux. I looked around for Linux apps that would let me play audio through a virtual mic, and found ["pipewire-soundpad"](https://github.com/arabianq/pipewire-soundpad) (cool software btw, you should check it out). Except it lacked a few features I would have liked:

- you had only one volume slider to control both the volume of the virtual mic and the volume for the self-playback,
- you had to deal with additional components like a user service/daemon,
- if the GUI quit the audio routing would stay intact -- which is meant to be a feature but I disliked it and would rather the routing be torn down on app quit,

and a few other tiny things like that.

I'd started to learn Rust around this time and it felt like the perfect opportunity to create something cool. As I was partway through building AudioPass, I noticed that there were some updates to pipewire-soundpad which fixed some of the stuff I mentioned above. Unfortunately, I was a few weeks late to seeing them and now I was a little unsure of all the work I'd put into the app. So I decided to expand the scope of the app a little bit, put audio file playback on pause (you'll find a bunch of `TODO-file-playback`s in the code lol) and added support for routing audio from locally running apps.

There's guaranteed to be software similar to AudioPass out there, especially on Linux (such as pipewire-soundpad itself), but my initial goal was to just learn and understand idiomatic Rust by building something cool. I feel like I've more or less achieved that over the last few months. Now my goal is to finish up the features I'd initially planned (things like audio file playback, hotkeys, audio normalization, possibly mixing physical mic input along with selected audio, etc.) and "complete" the app. In that sense, you could say the app is currently in beta or something.

# bloodthief-wasm-autosplitter

An autosplitter for Bloodthief written in Rust using the asr crate for LiveSplitOne - NOT LiveSplit

## Features:

The autosplitter:
- tracks your igt
- automatically starts the timer
- splits on:
  - checkpoint
  - key
  - first secret
  - finish


## Usage

Grab the latest release from [here](https://github.com/olvior/bloodthief-wasm-autosplitter/releases/latest)

I would recommend using [LiveSplitOne Druid by AlexKnauth](https://github.com/AlexKnauth/livesplit-one-druid/releases/latest)

For Linux you have to give livesplit the access to read other programs' memory. One of the ways to do that is running the command `sudo setcap CAP_SYS_PTRACE=+eip LiveSplitOne` in the same directory as the `LiveSplitOne` binary.

Make sure that you are on game time and not real time in livesplitone.

For more details look [here](https://github.com/AlexKnauth/hollowknight-autosplit-wasm?tab=readme-ov-file#installation)

NOTE: Windows isn't actually supported at the moment but it's something I'll try do later


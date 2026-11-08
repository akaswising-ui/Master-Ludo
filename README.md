# Master Ludo

A touch-friendly, offline Ludo board game written in Rust. The game uses Macroquad for rendering and input; the rules engine is kept separate in `src/lib.rs` so it can be tested independently.

## Included in this first version

- 2–4 local players (pass-and-play)
- Four tokens per player and a six to leave the yard
- Dice rolls, legal move selection, and automatic movement when only one token can move
- Token captures and protected safe squares
- Exact roll to reach home, extra turns, and the three-consecutive-sixes rule
- Win detection and quick restart
- Responsive board drawn directly from Rust code, with no image or JavaScript assets

## Run on desktop

Install the Rust toolchain, then run:

```sh
cargo run --release
```

Run the rules-engine tests with:

```sh
cargo test
```

## Android

Macroquad supports Android. Android packaging still requires the Android build toolchain and the platform setup described by the official Macroquad Android guide. This repository currently contains the Rust game source and Cargo manifest; it does not contain a signed APK.

## Current scope

This is the first playable offline foundation. Online matchmaking, accounts, chat, cloud saves, bots, sound effects, and network multiplayer are not included yet. The visuals are original and do not reuse Ludo King branding or artwork.

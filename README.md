# wolf3d_rs

[![Build Status](https://travis-ci.org/jfortin42/wolf3d_rs.svg)](https://travis-ci.com/jfortin42/wolf3d_rs)
[![License](https://img.shields.io/badge/license-Apache%202-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)

This is an implementation of Wolf3d in the [Rust Programming Language](https://www.rust-lang.org).

## preview

<img align="center"
src="https://github.com/jfortin42/wolf3d_rs/blob/master/screenshots/42_geometry_map.png" width="100%" height="544px" />
<br />
<br />
<img align="center"
src="https://github.com/jfortin42/scop/blob/master/screenshots/wolf3d_screen.png" width="100%" height="544px" />

## Build Instructions

Before building `wolf3d_rs`, you will need to install the developpement libraries for [SDL2](https://www.libsdl.org), preferably with the package manager that comes
with your operating system.

Example for Debian/Ubuntu:

    sudo apt-get install libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev

Example for Mac OSX

    brew install sdl2 sdl2_image sdl2_ttf

You might also like to read the README for these projects:

- <https://github.com/AngryLawyer/rust-sdl2>
- <https://github.com/xsleonard/rust-sdl2_image>
- <https://github.com/andelf/rust-sdl2_ttf>

To build `wolf3d_rs`, type the following commands:

    git clone https://github.com/jfortin42/wolf3d_rs
    cd wolf3d_rs
    cargo build --release

## How to Play

For a quick start, try this:

    cargo run --release -- microban.slc

- Use the arrow keys to move the player.
- Type `R` to retry the current level.
- Type `N` to skip the current level.

## Graphics Options

By default, the game will start in 1024x768 windowed mode.
You can modify the width and height of the window as well as switch to fullscreen mode.

Example:

    cargo run --release -- microban.slc --width=1920 --height=1080 --fullscreen

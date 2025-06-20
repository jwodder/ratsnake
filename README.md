[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![CI Status](https://github.com/jwodder/ratsnake/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/ratsnake/actions/workflows/test.yml) <!-- [![codecov.io](https://codecov.io/gh/jwodder/ratsnake/branch/main/graph/badge.svg)](https://codecov.io/gh/jwodder/ratsnake) -->
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.74-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/ratsnake.svg)](https://opensource.org/licenses/MIT)

`ratsnake` is an implementation of the video game [Snake][] in [Rust][] built
using the [Ratatui][] library.  Move the snake around with arrow keys,
<kbd>h</kbd>/<kbd>j</kbd>/<kbd>k</kbd>/<kbd>l</kbd>, or
<kbd>w</kbd>/<kbd>a</kbd>/<kbd>s</kbd>/<kbd>d</kbd> in order to eat the red
dots and grow — but don't collide with yourself!

[Snake]: https://en.wikipedia.org/wiki/Snake_(video_game_genre)
[Rust]: https://www.rust-lang.org
[Ratatui]: https://ratatui.rs

Installation
============

In order to install `ratsnake`, you first need to have [Rust and Cargo
installed](https://www.rust-lang.org/tools/install).  You can then build the
latest version of `ratsnake` and install it in `~/.cargo/bin` by running:

    cargo install --git https://github.com/jwodder/ratsnake

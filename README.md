[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![CI Status](https://github.com/jwodder/ratsnake/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/ratsnake/actions/workflows/test.yml)
[![codecov.io](https://codecov.io/gh/jwodder/ratsnake/branch/main/graph/badge.svg)](https://codecov.io/gh/jwodder/ratsnake)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.82-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/ratsnake.svg)](https://opensource.org/licenses/MIT)

`ratsnake` is an implementation of the video game [Snake][] in [Rust][] built
using the [Ratatui][] library.  Guide the snake around the level to eat fruits,
making it grow longer, but don't run into yourself!

[Snake]: https://en.wikipedia.org/wiki/Snake_(video_game_genre)
[Rust]: https://www.rust-lang.org
[Ratatui]: https://ratatui.rs

Installation
============

In order to install `ratsnake`, you first need to have [Rust and Cargo
installed](https://www.rust-lang.org/tools/install).  You can then build the
latest version of `ratsnake` and install it in `~/.cargo/bin` by running:

    cargo install --git https://github.com/jwodder/ratsnake

Usage
=====

Run `ratsnake` to bring up the program's main menu.  The `ratsnake` program
currently does not recognize any command-line options.

`ratsnake` is optimized for display on 80 column by 24 line terminals.  If your
terminal is bigger than this, the interface will be centered in the middle.  If
your terminal is smaller, you're going to have a bad time.

All screens in `ratsnake` support directional movement with the arrow keys,
<kbd>h</kbd>/<kbd>j</kbd>/<kbd>k</kbd>/<kbd>l</kbd>,
<kbd>w</kbd>/<kbd>a</kbd>/<kbd>s</kbd>/<kbd>d</kbd>, and
<kbd>2</kbd>/<kbd>4</kbd>/<kbd>6</kbd>/<kbd>8</kbd>.  In addition, pressing
<kbd>Ctrl</kbd>+<kbd>C</kbd> at any point during program execution will
immediately end the program.

Main Menu
---------

![Screenshot of the main menu](screenshots/mainmenu.png)

The main menu allows the user to configure various options for Snake before
starting a game.  The options are saved to a file (See "Data Directory" below)
that is loaded on program startup and updated before starting a new game.

The following options can be set:

- **Wraparound** — If this option is set, the borders of the game level will
  wrap around so that the snake can pass into one side and come out the
  opposite.  When this is not set, the snake will die upon coming into contact
  with a level border.

- **Obstacles** — If this option is set, random obstacles will be placed in the
  game level; coming into contact with one kills the snake.

- **Fruits** — Set the number of fruits present at all times.  May be any
  integer from 1 through 10.

- **Level Size** — Set the dimensions of the game level, choosing from small
  (38×8), medium (53×12), and large (76×19).

### Key Bindings

| Key                                                        | Command                                                |
| ---------------------------------------------------------- | ------------------------------------------------------ |
| <kbd>k</kbd>, <kbd>w</kbd>, <kbd>8</kbd>, <kbd>Up</kbd>    | Move up an item                                        |
| <kbd>j</kbd>, <kbd>s</kbd>, <kbd>2</kbd>, <kbd>Down</kbd>  | Move down an item                                      |
| <kbd>h</kbd>, <kbd>a</kbd>, <kbd>4</kbd>, <kbd>Left</kbd>  | Decrease or unset the current option                   |
| <kbd>l</kbd>, <kbd>d</kbd>, <kbd>6</kbd>, <kbd>Right</kbd> | Increase or set the current option                     |
| <kbd>Tab</kbd>                                             | Move down an item, circling around at the bottom       |
| <kbd>Shift</kbd>+<kbd>Tab</kbd>                            | Move up an item, circling around at the top            |
| <kbd>Home</kbd>                                            | Jump to the first item in the menu                     |
| <kbd>End</kbd>                                             | Jump to the last item in the menu                      |
| <kbd>Space</kbd>                                           | Toggle the current option                              |
| <kbd>Enter</kbd>                                           | Toggle the current option or select the current button |
| <kbd>p</kbd>                                               | Play a game of Snake                                   |
| <kbd>q</kbd>                                               | Quit                                                   |

Game
----

![Screenshot of a game](screenshots/game.png)

Upon selecting "Play" in the main menu, a game of Snake starts immediately with
the configured options.  The snake advances one cell at a time at regular
intervals; the directional keys can be used to change its direction of motion.
The goal is to guide the snake to consume fruits (red dots) that randomly
appear on the level; eating a fruit increases your score by 1 (displayed at the
top of the screen) but also makes the snake longer, making it harder to avoid
self-collisions.

Pressing <kbd>Escape</kbd> or defocusing the terminal during play will pause
the game.  While paused, a pop-up menu is displayed, giving you the choice of
resuming/unpausing, restarting the game using the same options (though
obstacles will be re-randomized), returning to the main menu, or quitting the
program.

The game lasts until the snake's head collides with its body, the level border
(if the wraparound option was not enabled), or an obstacle, all of which kill
the snake.  (If you're very skilled, you may also get a game over if you manage
to fill the level with the snake.)  When the game ends, a message is displayed,
and you can choose to start a new game with the same options (by pressing
<kbd>r</kbd>), return to the main menu (by pressing <kbd>m</kbd>), or quit the
program (by pressing <kbd>q</kbd>).

High scores are tracked and saved to a file automatically (See "Data Directory"
below).  Each combination of game options has its own separate high score.
Note that quitting a game in the middle of play will not cause a new high score
to be registered.

Data Directory
==============

Options and high scores are saved in a data directory in your home folder.  The
location of this directory depends on your OS:

- Linux — `~/.local/share/ratsnake/` or `$XDG_DATA_HOME/ratsnake/`
- macOS — `~/Library/Application Support/ratsnake/`
- Windows — `%USERPROFILE%\AppData\Local\ratsnake\`

Acknowledgements
================

Much of `ratsnake`'s functionality was inspired by
[`nsnake`](https://github.com/alexdantas/nSnake) by Alexandre Dantas (no longer
maintained).

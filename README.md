# `web-bg`

Interactive website backgrounds in [Rust](https://www.rust-lang.org/) with [Bevy](https://bevyengine.org/).

Try it out locally with `cargo run` or see below for more info.

## Building

To build and run `web-bg` as a regular application for local testing, run [`cargo run --features dynamic,debug`](https://doc.rust-lang.org/cargo/commands/cargo-run.html). Note that while the first compilation will take a few minutes, subsequent builds should be much faster. Profiling support for [Tracy](https://github.com/wolfpld/tracy) can be enabled by adding the `profile` feature ([`cargo run --features dynamic,profile`](https://doc.rust-lang.org/cargo/commands/cargo-run.html)).

To build `web-bg` as a regular application, run [`cargo build --release`](https://doc.rust-lang.org/cargo/commands/cargo-build.html). The compiled binary will be located in `./target/release/web-bg[.exe]`. This build takes a few minutes, and is not recommended for debugging/testing/development.

To build `web-bg` for the web (with full optimizations), run [`cargo build --profile release-wasm --target wasm32-unknown-unknown`](https://doc.rust-lang.org/cargo/commands/cargo-build.html), create a new directory named `web` (`mkdir web`), then run [`wasm-bindgen --out-name web --out-dir target/wasm --target web target/wasm32-unknown-unknown/release-wasm/web-bg.wasm`](https://github.com/rustwasm/wasm-bindgen) and optionally [`wasm-opt -Oz --output web/web_bg.wasm target/wasm/web_bg.wasm`](https://github.com/WebAssembly/binaryen), then copy `index.html` and `target/wasm/web.js` as `background.js` into it (`cp index.html web/index.html` and `cp target/wasm/web.js web/background.js`). This build takes a few minutes, and is not recommended for debugging/testing/development.

### Web builds

To try out `web-bg` in a web browser follow the instructions above to build it for the web, start an HTTP server (e.g. with `python -m http.server -d web`) and open [`http://localhost:8000/`](http://localhost:8000/) in a browser.

When deploying `web-bg` on a website, serve the generated `.js` and `.wasm` files and add the appropriate elements to your website. You can use `index.html` as a template. See `.github/workflows/build.yaml` for an example of an automatic build of `web-bg`.

## Minigames/backgrounds

Each minigame has one [Cargo feature](https://doc.rust-lang.org/cargo/reference/features.html) which controls whether that minigame will be included in the final bundle. By default, all minigames are included.

|       Title |       Feature | Done? | Description |
| ----------- | ------------- | ----- | ----------- |
|   Asteroids |   `asteroids` |    No | An *[Asteroids](https://en.wikipedia.org/wiki/Asteroids_(video_game))*-inspired space flying game. |
|      Lander |      `lander` |    No | A 2.5D/2D moon landing simulator in the [Lunar Lander](https://en.wikipedia.org/wiki/Lunar_Lander_(video_game_genre)) genre, inspired by [XKCD 2712 - *Gravity*](https://xkcd.com/2712/). |
|      Mapgen |      `mapgen` |    No | [Wavefunction collapse](https://robertheaton.com/2018/12/17/wavefunction-collapse-algorithm/)-based [Carcassonne](https://en.wikipedia.org/wiki/Carcassonne_(board_game))-esque map generator. |
|     MAP-MAN |      `mapman` |    No | A 2.5D *[PAC-MAN](https://en.wikipedia.org/wiki/Pac-Man)*-based game played on a real-world map, similar to [Google Maps' 2015 and 2017 April Fool's jokes](https://pacman.fandom.com/wiki/Google_Maps_Pac-Man), but with 3D buildings. |
|   Maze Cave |        `maze` |   Yes | A randomly generated maze/cave. |
|     Portoom |     `portoom` |    No | A *[Doom](https://en.wikipedia.org/wiki/Doom_(1993_video_game))*-style, *[Portal](https://en.wikipedia.org/wiki/Portal_(video_game))*-inspired first-person shooter. |
|     Racecar |     `racecar` |    No | A 2D multiplayer [slot car racing](https://en.wikipedia.org/wiki/Slot_car_racing) game. Real multiplayer support (playing against other people) coming soon. |

## Usage on the web

See `index.html` for an example of usage.

`web-bg` needs a `canvas` element with id `background` to render to.
The size of that element will be set to match the size of its parent by `web-bg`.

`web-bg` takes keyboard, mouse, and touchscreen input from its canvas element.
Websites should provide a way for the user to focus on that element, for example by clicking/tapping on it or via a global keyboard shortcut.

`web-bg` dispatches JavaScript events to the `window` during various phases of execution:

- `web-bg-load` when the application starts executing
- `web-bg-init` when the application has initialized
- `web-bg-start` when the application is fully ready for usage (`web-bg`'s canvas should be hidden until this event is received)
- `web-bg-panic` if the application panics (`web-bg`'s canvas should be hidden when this event is received)

### Logging on the web

If the `console_log` feature is enabled and you compile `web-bg` for the web, log messages will be logged to the console and tracing spans will be measured using the Performance API, at the expense of degraded application performance.
The `console_log` feature does nothing when *not* compiling for the web.

## Attribution

In addition to Cargo dependencies, the following additional resources are used as part of `web-bg`:

- Maze Cave (in `assets/maze/`):
  - Player character based on ["Reaper" by SamuelLee](https://samuellee.itch.io/reaper-animated-pixel-art) (`player-idle.png` and `player-walking.png`)
  - Player's torch from ["Cave Explorer" by SamuelLee](https://samuellee.itch.io/cave-explorer-animated-pixel-art) (`player-idle.png` and `player-walking.png`)
  - Cave tiles based on ["Textures" by PiiiXL](https://piiixl.itch.io/textures) (`cave-floor-1.png`, `cave-floor-2.png`, and `cave-wall.png`)
  - Food from ["Pixel Food" by ghostpixxells](https://ghostpixxells.itch.io/pixelfood) (`food.png` and `plate.png`)
- Miscellaneous:
  - The [Roboto font](https://fonts.google.com/specimen/Roboto), used under the terms of the [Apache 2.0 license](https://www.apache.org/licenses/LICENSE-2.0) in `assets/fonts/roboto.ttf` and `assets/fonts/roboto-bold.ttf`
  - The [Retro Pixel Thick font](https://retro-pixel-font.takwolf.com/), used under the terms of the [Open Font License version 1.1](https://raw.githubusercontent.com/TakWolf/retro-pixel-font/0e90d12/LICENSE-OFL) in `assets/fonts/pixel.ttf`
  - [`github-markdown-css`](https://github.com/sindresorhus/github-markdown-css), used under the terms of [the MIT license](./about.hbs#this-document) for styling in `about.hbs` (and the html file generated from it)

For Cargo dependencies, you can use [`cargo about generate -o ATTRIBUTION.html --all-features about.hbs`](https://github.com/EmbarkStudios/cargo-about) to generate an html file with information about dependencies and their licenses.

## License

This project (with the exception of some assets in the `assets` directory) is licensed under either of

- Apache license, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `web-bg` by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

# `web-bg`

Interactive website backgrounds in [Rust](https://www.rust-lang.org/) with [Bevy](https://bevyengine.org/).

Try it out locally with `cargo run` or see below for more info.

## Building

To build and run `web-bg` as a regular application for local testing, run [`cargo run --features debug`](https://doc.rust-lang.org/cargo/commands/cargo-run.html). Note that while the first compilation will take a few minutes, subsequent builds should only take a few seconds.

To build `web-bg` as a regular application, run [`cargo build --release`](https://doc.rust-lang.org/cargo/commands/cargo-build.html). The compiled binary will be located in `./target/release/web-bg{.exe}`. This build takes a few minutes, and is not recommended for debugging/testing/development.

To build `web-bg` for the web for testing or debugging (without many optimizations), run [`cargo build --target wasm32-unknown-unknown --features debug`](https://doc.rust-lang.org/cargo/commands/cargo-build.html) followed by [`wasm-bindgen --out-name web-bg --out-dir target/wasm-debug --target web target/wasm32-unknown-unknown/debug/web-bg.wasm`](https://github.com/rustwasm/wasm-bindgen). The compiled files will be located in `./target/wasm-debug/`. Note that while the first compilation will take a few minutes (because dependencies *do* get optimized), subsequent builds should only take a few seconds (as long as the dependencies stay the same and only `web-bg`'s code is changed).

To build `web-bg` for release on the web, with full optimizations, run [`cargo build --profile release-wasm --target wasm32-unknown-unknown`](https://doc.rust-lang.org/cargo/commands/cargo-build.html) followed by [`wasm-bindgen --out-name web-bg --out-dir target/wasm --target web target/wasm32-unknown-unknown/release-wasm/web-bg.wasm`](https://github.com/rustwasm/wasm-bindgen) and optionally [`wasm-opt -Oz --output target/wasm/web-bg.opt.wasm target/wasm/web-bg_bg.wasm`](https://github.com/WebAssembly/binaryen) then rename `target/wasm/web-bg.opt.wasm` to `target/wasm/web-bg_bg.wasm`. This build takes a few minutes, and is not recommended for debugging/testing/development.

### Web builds

To try out `web-bg` in a web browser follow the instructions above to build it for the web (with or without optimizations), start an HTTP server (e.g. with `python -m http.server`) and open [`http://localhost:8000/debug.html`](http://localhost:8000/debug.html) in a browser.

When deploying `web-bg` on a website, serve the generated `.js` and `.wasm` files and add the appropriate elements to your website. You can use `index.html` as a template. See `.github/workflows/build.yaml` for an example of an automatic build of `web-bg`.

## Minigames/backgrounds

Each minigame has one [Cargo feature](https://doc.rust-lang.org/cargo/reference/features.html) which controls whether that minigame will be included in the final bundle. By default, all minigames are included.

|       Title |       Feature | Done? | Description |
| ----------- | ------------- | ----- | ----------- |
|   Asteroids |   `asteroids` |    No | An *[Asteroids](https://en.wikipedia.org/wiki/Asteroids_(video_game))*-inspired space flying game. |
|   Maze Game |        `maze` |    No | A randomly generated infinite maze game with stylistic raycasting. |
|     Portoom |     `portoom` |    No | A *[Doom](https://en.wikipedia.org/wiki/Doom_(1993_video_game))*-style, *[Portal](https://en.wikipedia.org/wiki/Portal_(video_game))*-inspired first-person shooter. |
|     Racecar |     `racecar` |    No | A 2D multiplayer [slot car racing](https://en.wikipedia.org/wiki/Slot_car_racing) game. Real multiplayer support (playing against other people) coming soon. |
|      Lander |      `lander` |    No | A 2.5D/2D moon landing simulator in the [Lunar Lander](https://en.wikipedia.org/wiki/Lunar_Lander_(video_game_genre)) genre, inspired by [XKCD 2712 - *Gravity*](https://xkcd.com/2712/). |
| Astroguessr | `astroguessr` |    No | A space-based version of *[Geoguessr](https://www.geoguessr.com/)*. |
|     MAP-MAN |      `mapman` |    No | A 2.5D *[PAC-MAN](https://en.wikipedia.org/wiki/Pac-Man)*-based game played on a real-world map, similar to [Google Maps' 2015 and 2017 April Fool's jokes](https://pacman.fandom.com/wiki/Google_Maps_Pac-Man), but with 3D buildings. |

## Attribution

In addition to Cargo dependencies, the following open-source resources are used as part of `web-bg`:

- The [Roboto font](https://fonts.google.com/specimen/Roboto), used under the terms of the [Apache 2.0 license](./licenses/roboto.txt) in `assets/fonts/roboto.ttf` and `assets/fonts/roboto-bold.ttf`
- [`github-markdown-css`](https://github.com/sindresorhus/github-markdown-css), used under the terms of [the MIT license](./about.hbs#this-document) for styling in `about.hbs` (and the html file generated from it)

For Cargo dependencies, you can use [`cargo about generate -o ATTRIBUTION.html --all-features about.hbs`](https://github.com/EmbarkStudios/cargo-about) to generate an html file with information about dependencies and their licenses.

## License

This project is licensed under either of

- Apache license, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `web-bg` by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

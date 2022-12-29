# `web-bg`

Interactive website backgrounds in [Rust](https://www.rust-lang.org/) with [Bevy](https://bevyengine.org/).

Try it out locally with `cargo run` or see below for more info.

## Building

**TODO: build instructions.**

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

## License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   <https://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
   <https://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `web-bg` by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

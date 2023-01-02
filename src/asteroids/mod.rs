//! An *[Asteroids](https://en.wikipedia.org/wiki/Asteroids_(video_game))*-inspired space flying game.

use bevy::prelude::*;

pub fn systems() -> SystemSet {
	SystemSet::new().label("asteroids systems")
}

pub fn startup_systems() -> SystemSet {
	SystemSet::new().label("asteroids startup systems")
}

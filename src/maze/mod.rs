//! A randomly generated infinite maze game with stylistic raycasting.

use bevy::prelude::*;

pub fn systems() -> SystemSet {
	SystemSet::new().label("maze systems")
}

pub fn startup_systems() -> SystemSet {
	SystemSet::new().label("maze startup systems")
}

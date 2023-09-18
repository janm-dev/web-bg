//! A randomly generated maze/cave.

use bevy::prelude::*;

use crate::util::{input, PlayerInput};

#[allow(clippy::module_inception)]
mod maze;
mod player;

pub fn start(app: &mut App) {
	app.add_systems(
		Startup,
		(
			player::initialize,
			maze::initialize,
			|mut commands: Commands| {
				commands.spawn(Camera2dBundle::default());
			},
		),
	);

	app.add_systems(PreUpdate, input);

	app.add_systems(
		Update,
		(
			camera_movement,
			player::animation,
			player::movement,
			player::collision.after(player::movement),
			maze::spawn_visible_tiles,
			maze::despawn_invisible_tiles,
		),
	);
	app.insert_resource(PlayerInput::default());
}

fn camera_movement(
	mut camera: Query<&mut Transform, (With<Camera>, Without<player::Player>)>,
	player: Query<&Transform, (With<player::Player>, Without<Camera>)>,
	window: Query<&Window>,
) {
	/// The free movement space on each side of the screen as a proportion of
	/// the width/height of the screen
	const FREE_MOVEMENT_SPACE_PROPORTION: f32 = 0.2;

	let mut camera = camera
		.get_single_mut()
		.expect("there is more than one camera");
	let player = player.get_single().expect("there is more than one player");
	let window = window.get_single().expect("there is more than one window");

	let (width, height) = (
		window.width() * FREE_MOVEMENT_SPACE_PROPORTION,
		window.height() * FREE_MOVEMENT_SPACE_PROPORTION,
	);
	let player_displacement = player.translation - camera.translation;

	let deadzoned_displacement_x = player_displacement.x.abs() - width;
	let deadzoned_displacement_x = if deadzoned_displacement_x.is_sign_negative() {
		0.0
	} else {
		deadzoned_displacement_x
	};
	let deadzoned_displacement_x = deadzoned_displacement_x.copysign(player_displacement.x);

	let deadzoned_displacement_y = player_displacement.y.abs() - height;
	let deadzoned_displacement_y = if deadzoned_displacement_y.is_sign_negative() {
		0.0
	} else {
		deadzoned_displacement_y
	};
	let deadzoned_displacement_y = deadzoned_displacement_y.copysign(player_displacement.y);

	camera.translation += Vec3 {
		x: deadzoned_displacement_x,
		y: deadzoned_displacement_y,
		z: 0.0,
	};
}

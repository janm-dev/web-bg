//! A randomly generated maze/cave.

use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*, window::PrimaryWindow};

use crate::util::{input, PlayerInput};

mod food;
#[allow(clippy::module_inception)]
mod maze;
mod player;

pub fn start(app: &mut App) {
	app.add_systems(
		Startup,
		(player::initialize, maze::initialize, camera_initialization),
	);

	app.add_systems(PreUpdate, input);

	app.add_systems(
		Update,
		(
			camera_movement,
			player::animation,
			player::light_flicker,
			player::movement,
			player::collision.after(player::movement),
			maze::spawn_visible_tiles,
			maze::despawn_invisible_tiles,
			food::eat,
			food::dim,
		),
	);
	app.insert_resource(PlayerInput::default());
}

#[derive(Debug, Component)]
struct Target;

fn camera_initialization(mut commands: Commands) {
	commands.spawn(Camera2dBundle {
		camera: Camera {
			order: 1,
			..default()
		},
		camera_2d: Camera2d {
			clear_color: ClearColorConfig::None,
		},
		..default()
	});

	commands.spawn((
		Camera3dBundle {
			camera: Camera {
				order: 0,
				..default()
			},
			projection: Projection::Orthographic(OrthographicProjection::default()),
			transform: Transform {
				translation: Vec3 {
					x: 0.0,
					y: 0.0,
					z: 1.0,
				},
				..default()
			},
			..default()
		},
		ComputedVisibility::default(),
	));
}

fn camera_movement(
	mut cameras: Query<&mut Transform, (With<Camera>, Without<player::Player>)>,
	player: Query<&Transform, With<player::Player>>,
	window: Query<&Window, With<PrimaryWindow>>,
) {
	/// The free movement space on each side of the screen as a proportion of
	/// the width/height of the screen
	const FREE_MOVEMENT_SPACE_PROPORTION: f32 = 0.2;

	for mut camera in &mut cameras {
		let player = player.single();
		let window = window.single();

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
}

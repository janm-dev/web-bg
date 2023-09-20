//! `web-bg` utilities and other miscellaneous things.

use bevy::prelude::*;

/// Quickly declare minigames
///
/// Only use this macro once. It creates a const called `GAMES` with a name and
/// [`SystemsFn`]s, the first for startup systems, the second for regular
/// systems. This macro handles cfg features and module declarations.
#[macro_export]
macro_rules! games {
	{$($feat:literal => $name:ident),*$(,)?} => {
		$(
			#[cfg(feature = $feat)] mod $name;
		)*

		struct Game {
			name: &'static str,
			start: fn(&mut bevy::app::App)
		}

		const GAMES: &[Game] = &[
			$(
				#[cfg(feature = $feat)] Game { name: $feat, start: $name::start },
			)*
		];

		$(
			#[cfg(not(feature = $feat))]
		)*
		compile_error!("At least one minigame must be enabled");
	}
}

/// Up/down/left/right movement input within the range from `-1.0` to `1.0`
///
/// If the input for either axis is within the deadzone, it is set to exactly
/// `0.0`
#[derive(Debug, Clone, Copy, Resource, Default)]
pub struct PlayerInput {
	pub up: f32,
	pub right: f32,
}

impl PlayerInput {
	/// Whether there is any input
	#[must_use]
	pub fn is_moving(self) -> bool {
		self.up != 0.0 || self.right != 0.0
	}
}

impl From<PlayerInput> for Vec2 {
	fn from(PlayerInput { up, right }: PlayerInput) -> Self {
		Self { x: right, y: up }
	}
}

impl From<PlayerInput> for Vec3 {
	fn from(PlayerInput { up, right }: PlayerInput) -> Self {
		Self {
			x: right,
			y: up,
			z: 0.0,
		}
	}
}

/// A system for processing up/down/left/right movement input, shared across
/// games
///
/// # Usage
///
/// Insert the [`PlayerInput`] resource into the app on startup (this is not
/// done automatically), then register this system, ideally before any
/// movement/animation processing (e.g. in the `PreUpdate`) schedule
pub fn input(
	mut input: ResMut<PlayerInput>,
	key_input: Res<Input<KeyCode>>,
	gamepads: Res<Gamepads>,
	pad_input: Res<Input<GamepadButton>>,
	stick_input: Res<Axis<GamepadAxis>>,
) {
	const DEADZONE: f32 = 0.05;

	let mut up = 0.0;
	let mut right = 0.0;

	// Keyboard WASD
	if key_input.pressed(KeyCode::W) {
		up += 1.0;
	}

	if key_input.pressed(KeyCode::S) {
		up -= 1.0;
	}

	if key_input.pressed(KeyCode::D) {
		right += 1.0;
	}

	if key_input.pressed(KeyCode::A) {
		right -= 1.0;
	}

	// Keyboard arrow keys
	if key_input.pressed(KeyCode::Up) {
		up += 1.0;
	}

	if key_input.pressed(KeyCode::Down) {
		up -= 1.0;
	}

	if key_input.pressed(KeyCode::Right) {
		right += 1.0;
	}

	if key_input.pressed(KeyCode::Left) {
		right -= 1.0;
	}

	for gamepad in gamepads.iter() {
		// Gamepad buttons
		if pad_input.pressed(GamepadButton {
			gamepad,
			button_type: GamepadButtonType::DPadUp,
		}) {
			up += 1.0;
		}

		if pad_input.pressed(GamepadButton {
			gamepad,
			button_type: GamepadButtonType::DPadDown,
		}) {
			up -= 1.0;
		}

		if pad_input.pressed(GamepadButton {
			gamepad,
			button_type: GamepadButtonType::DPadRight,
		}) {
			right += 1.0;
		}

		if pad_input.pressed(GamepadButton {
			gamepad,
			button_type: GamepadButtonType::DPadLeft,
		}) {
			right -= 1.0;
		}

		// Gamepad stick
		if let Some(i) = stick_input.get(GamepadAxis {
			gamepad,
			axis_type: GamepadAxisType::LeftStickY,
		}) {
			if i.abs() > DEADZONE {
				up += i;
			}
		}

		if let Some(i) = stick_input.get(GamepadAxis {
			gamepad,
			axis_type: GamepadAxisType::LeftStickX,
		}) {
			if i.abs() > DEADZONE {
				right += i;
			}
		}
	}

	let up = if up.abs() > DEADZONE { up } else { 0.0 };
	let right = if right.abs() > DEADZONE { right } else { 0.0 };

	*input = PlayerInput {
		up: up.clamp(-1.0, 1.0),
		right: right.clamp(-1.0, 1.0),
	}
}

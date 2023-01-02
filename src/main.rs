#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::needless_pass_by_value)] // A bunch of Bevy things require this
#![allow(clippy::module_name_repetitions)]

pub mod util;

#[cfg(feature = "debug")]
use bevy::{
	diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
	log::Level,
	window::close_on_esc,
};
use bevy::{
	log::LogPlugin,
	prelude::*,
	render::settings::{WgpuSettings, WgpuSettingsPriority},
};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use rand::seq::SliceRandom;

games! {
	"asteroids" => asteroids,
	"maze" => maze,
	// "portoom" => portoom,
	// "racecar" => racecar,
	// "lander" => lander,
	// "astroguessr" => astroguessr,
	// "mapman" => mapman,
}

#[bevy_main]
pub fn main() {
	#[cfg(target_arch = "wasm32")]
	console_error_panic_hook::set_once();

	let game = GAMES
		.choose(&mut rand::thread_rng())
		.expect("there are no games");

	println!("Starting game \"{}\"", game.0);

	start_app()
		.add_startup_system_set(game.1())
		.add_system_set(game.2())
		.run();
}

fn start_app() -> App {
	let mut app = App::new();

	let default_plugins = DefaultPlugins
		.set(WindowPlugin {
			window: WindowDescriptor {
				canvas: cfg!(target_arch = "wasm32").then(|| "#background".to_string()),
				fit_canvas_to_parent: true,
				..default()
			},
			..default()
		})
		.set(ImagePlugin::default_nearest())
		.set(AssetPlugin {
			watch_for_changes: cfg!(feature = "debug"),
			..default()
		})
		.add_before::<AssetPlugin, _>(EmbeddedAssetPlugin)
		.disable::<LogPlugin>();

	#[cfg(feature = "debug")]
	let default_plugins = default_plugins.set(AssetPlugin {
		watch_for_changes: true,
		..default()
	});

	app.insert_resource(WgpuSettings {
		priority: WgpuSettingsPriority::Compatibility,
		..default()
	})
	.insert_resource(ClearColor(Color::NONE))
	.insert_resource(Msaa { samples: 4 })
	.add_plugins(default_plugins);

	#[cfg(feature = "debug")]
	{
		app.add_plugin(LogPlugin {
			level: Level::DEBUG,
			..default()
		});
		app.add_plugin(LogDiagnosticsPlugin::default());
		app.add_plugin(FrameTimeDiagnosticsPlugin);
		app.add_system(close_on_esc);
	}

	app.add_startup_system(setup);

	app
}

/// Common setup for all minigames
fn setup(mut windows: ResMut<Windows>) {
	let window = windows.primary_mut();
	window.set_title("web-bg".to_string());
	window.set_maximized(true);
	window.set_resizable(true);
}

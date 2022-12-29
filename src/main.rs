#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::needless_pass_by_value)] // A bunch of Bevy things require this
#![allow(clippy::module_name_repetitions)]

use bevy::{
	prelude::*,
	render::settings::{WgpuSettings, WgpuSettingsPriority},
};
use bevy_embedded_assets::EmbeddedAssetPlugin;

#[bevy_main]
pub fn main() {
	#[cfg(target_arch = "wasm32")]
	console_error_panic_hook::set_once();

	let mut app = App::new();

	app.insert_resource(WgpuSettings {
		priority: WgpuSettingsPriority::Compatibility,
		..default()
	})
	.insert_resource(ClearColor(Color::NONE))
	.insert_resource(Msaa { samples: 4 })
	.add_plugins(
		DefaultPlugins
			.build()
			.add_before::<AssetPlugin, _>(EmbeddedAssetPlugin)
			.set(ImagePlugin::default_nearest()),
	);

	#[cfg(feature = "debug")]
	app.add_system(bevy::window::close_on_esc);

	app.add_startup_system(setup);

	// TODO: start random game

	app.run();
}

fn setup(mut windows: ResMut<Windows>) {
	let window = windows.primary_mut();
	window.set_title("web-bg".to_string());
	window.set_maximized(true);
	window.set_resizable(true);
}

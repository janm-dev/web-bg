#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::needless_pass_by_value)] // A bunch of Bevy things require this
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::tabs_in_doc_comments)]

pub mod util;

use std::time::Duration;
#[cfg(target_arch = "wasm32")]
use std::{
	backtrace::{Backtrace, BacktraceStatus},
	panic::PanicInfo,
};

use bevy::{
	asset::ChangeWatcher,
	log::LogPlugin,
	prelude::*,
	window::{WindowMode, WindowResolution},
};
#[cfg(feature = "debug")]
use bevy::{
	diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
	log::Level,
	window::close_on_esc,
};
#[cfg(feature = "debug")]
use bevy_debug_text_overlay::OverlayPlugin;
use bevy_embedded_assets::EmbeddedAssetPlugin;
#[cfg(feature = "debug")]
use bevy_screen_diagnostics::{
	ScreenDiagnosticsPlugin, ScreenEntityDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin,
};
use rand::seq::SliceRandom;

games! {
	// "asteroids" => asteroids,
	// "lander" => lander,
	// "mapgen" => mapgen,
	// "mapman" => mapman,
	"maze" => maze,
	// "portoom" => portoom,
	// "racecar" => racecar,
}

#[cfg(target_arch = "wasm32")]
fn panic_hook(panic_info: &PanicInfo<'_>) {
	use wasm_bindgen::prelude::*;

	#[wasm_bindgen]
	extern "C" {
		#[wasm_bindgen(js_namespace = console)]
		fn error(msg: String);

		fn web_bg_panic(msg: String);

		type Error;

		#[wasm_bindgen(constructor)]
		fn new() -> Error;

		#[wasm_bindgen(structural, method, getter)]
		fn stack(error: &Error) -> String;
	}

	let mut msg = panic_info.to_string();

	msg.push_str("\n\nJS/WASM Stack:\n\n");
	let e = Error::new();
	let stack = e.stack();
	msg.push_str(&stack);

	let stack = Backtrace::force_capture();
	if stack.status() == BacktraceStatus::Captured {
		msg.push_str("\n\nRust Stack:\n\n");
		msg.push_str(&stack.to_string());
	}

	msg.push_str("\n\n");
	error(msg.clone());
	web_bg_panic(msg);
}

#[bevy_main]
#[allow(clippy::missing_panics_doc)]
pub fn main() {
	#[cfg(feature = "debug")]
	util::init_startup_measurement();

	#[cfg(target_arch = "wasm32")]
	std::panic::set_hook(Box::new(panic_hook));

	let game = GAMES
		.choose(&mut rand::thread_rng())
		.expect("there are no games");

	info!("Starting game \"{}\"", game.name);

	let mut app = App::new();

	let default_plugins = DefaultPlugins
		.set(WindowPlugin {
			primary_window: Some(Window {
				mode: WindowMode::BorderlessFullscreen,
				resizable: true,
				fit_canvas_to_parent: true,
				canvas: cfg!(target_arch = "wasm32").then(|| "#background".to_string()),
				title: if cfg!(target_arch = "wasm32") {
					""
				} else {
					"web-bg"
				}
				.to_string(),
				..default()
			}),
			..default()
		})
		.set(ImagePlugin::default_nearest())
		.set(AssetPlugin {
			watch_for_changes: cfg!(feature = "debug").then_some(ChangeWatcher {
				delay: Duration::from_millis(500),
			}),
			..default()
		})
		.add_before::<AssetPlugin, _>(EmbeddedAssetPlugin)
		.disable::<LogPlugin>();

	app.insert_resource(ClearColor(Color::NONE))
		.insert_resource(Msaa::Sample4)
		.add_plugins(default_plugins);

	#[cfg(feature = "debug")]
	{
		app.add_plugins((
			LogPlugin {
				level: Level::DEBUG,
				..default()
			},
			LogDiagnosticsPlugin::default(),
			FrameTimeDiagnosticsPlugin,
			OverlayPlugin {
				font_size: 16.0,
				..default()
			},
			ScreenDiagnosticsPlugin::default(),
			ScreenFrameDiagnosticsPlugin,
			ScreenEntityDiagnosticsPlugin,
		));
		app.add_systems(Startup, util::initial_startup_measurement);
		app.add_systems(Update, (close_on_esc, util::full_startup_measurement));
	}

	(game.start)(&mut app);
	app.run();
}

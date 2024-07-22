#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::needless_pass_by_value)] // A bunch of Bevy things require this
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::tabs_in_doc_comments)]

pub mod events;
pub mod util;

use std::backtrace::{Backtrace, BacktraceStatus};
#[allow(deprecated)] // PanicHookInfo is not stable yet
use std::panic::PanicInfo;

#[cfg(any(feature = "debug", not(target_arch = "wasm32")))]
use bevy::window::close_on_esc;
#[cfg(feature = "debug")]
use bevy::{
	diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
	log::Level,
};
use bevy::{log::LogPlugin, prelude::*, window::WindowMode};
#[cfg(feature = "debug")]
use bevy_debug_text_overlay::OverlayPlugin;
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
#[cfg(feature = "debug")]
use bevy_screen_diagnostics::{
	ScreenDiagnosticsPlugin, ScreenEntityDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin,
};
#[cfg(all(target_arch = "wasm32", not(target_feature = "atomics")))]
use rlsf::SmallGlobalTlsf;
#[cfg(all(feature = "console_log", target_arch = "wasm32"))]
use tracing_subscriber::{fmt::format::Pretty, prelude::*};
#[cfg(all(feature = "console_log", target_arch = "wasm32"))]
use tracing_web::{performance_layer, MakeConsoleWriter};
use util::{Rand, TurboRand};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(all(target_arch = "wasm32", not(target_feature = "atomics")))]
#[global_allocator]
static ALLOC: SmallGlobalTlsf = SmallGlobalTlsf::new();

games! {
	// "asteroids" => asteroids,
	// "lander" => lander,
	// "mapgen" => mapgen,
	// "mapman" => mapman,
	"maze" => maze,
	// "portoom" => portoom,
	// "racecar" => racecar,
}

fn panic_hook(panic_info: &PanicInfo<'_>) {
	#[cfg(target_arch = "wasm32")]
	#[wasm_bindgen]
	extern "C" {
		type Error;

		#[wasm_bindgen(constructor)]
		fn new() -> Error;

		#[wasm_bindgen(structural, method, getter)]
		fn stack(error: &Error) -> String;
	}

	let mut msg = panic_info.to_string();

	#[cfg(target_arch = "wasm32")]
	{
		msg.push_str("\n\nJS/WASM Stack:\n\n");
		let e = Error::new();
		let stack = e.stack();
		msg.push_str(&stack);
	}

	let stack = Backtrace::force_capture();
	if stack.status() == BacktraceStatus::Captured {
		msg.push_str("\n\nRust Stack:\n\n");
		msg.push_str(&stack.to_string());
	}

	msg.push_str("\n\n");

	#[cfg(target_arch = "wasm32")]
	web_sys::console::error_1(&JsValue::from_str(&msg));

	events::panic(msg);
}

#[bevy_main]
#[allow(clippy::missing_panics_doc)]
pub fn main() {
	std::panic::set_hook(Box::new(panic_hook));

	#[cfg(all(feature = "console_log", target_arch = "wasm32"))]
	{
		let fmt_layer = tracing_subscriber::fmt::layer()
			.with_ansi(false)
			.with_timer(util::PerformanceTimer)
			.with_writer(MakeConsoleWriter);

		let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

		tracing_subscriber::registry()
			.with(fmt_layer.with_filter(util::LogFilter))
			.with(perf_layer.with_filter(util::LogFilter))
			.init();
	}

	events::init();

	let rng = Rand::new();
	let game = rng.sample(GAMES).expect("there are no games");

	events::loaded(game.name);

	let mut app = App::new();

	let default_plugins = DefaultPlugins
		.set(WindowPlugin {
			primary_window: Some(Window {
				mode: WindowMode::BorderlessFullscreen,
				resizable: true,
				canvas: cfg!(target_arch = "wasm32").then(|| "#background".to_string()),
				title: if cfg!(target_arch = "wasm32") {
					String::new()
				} else {
					format!("{} | web-bg", game.name)
				},
				..default()
			}),
			..default()
		})
		.set(ImagePlugin::default_nearest())
		.set(AssetPlugin::default())
		.add_before::<AssetPlugin, _>(EmbeddedAssetPlugin {
			mode: PluginMode::ReplaceDefault,
		})
		.disable::<LogPlugin>();

	app.insert_resource(ClearColor(Color::NONE))
		.insert_resource(rng)
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
	}

	#[cfg(any(feature = "debug", not(target_arch = "wasm32")))]
	app.add_systems(Update, close_on_esc);

	app.add_systems(PostStartup, events::initialized);
	app.add_systems(Update, events::started);

	(game.start)(&mut app);

	app.run();
}

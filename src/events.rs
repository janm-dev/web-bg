//! `web-bg`-generated JavaScript events

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
use std::{
	fmt::{Display, Formatter, Result as FmtResult},
	sync::{Once, OnceLock},
	time::Duration,
};

use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::{CustomEvent, CustomEventInit, Event};
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

static STARTUP_TIME: OnceLock<Instant> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunEvent {
	/// The application loaded (near the beginning of `main`)
	///
	/// Contains the name of the game which is being loaded
	Loaded(&'static str),
	/// The application has initialized (after the startup systems have run)
	///
	/// Contains the time since application startup
	Initialized(Option<Duration>),
	/// The application started (after at least one frame was rendered and the
	/// application is ready to be used)
	///
	/// Contains the time since application startup
	Started(Option<Duration>),
	/// The application panicked (a rust panic happened)
	///
	/// Contains panic information as a string
	///
	/// This event is also dispatched on wasm if `main` returns
	Panicked(Option<String>),
}

impl RunEvent {
	/// Get the name of this event
	///
	/// On `wasm32` this is the `type` of the `CustomEvent` for this `RunEvent`
	#[must_use]
	pub const fn name(&self) -> &'static str {
		match self {
			Self::Loaded(_) => "web-bg-load",
			Self::Initialized(_) => "web-bg-init",
			Self::Started(_) => "web-bg-start",
			Self::Panicked(_) => "web-bg-panic",
		}
	}

	/// Get the details of this `RunEvent` as a JS value (type depending on the
	/// `RunEvent` variant)
	#[cfg(target_arch = "wasm32")]
	#[must_use]
	pub fn details(&self) -> JsValue {
		match self {
			Self::Loaded(s) => JsValue::from_str(s),
			Self::Initialized(None) | Self::Started(None) | Self::Panicked(None) => JsValue::null(),
			Self::Initialized(Some(d)) | Self::Started(Some(d)) => {
				JsValue::from_f64(d.as_secs_f64())
			}
			Self::Panicked(Some(s)) => JsValue::from_str(s),
		}
	}

	/// Get a JavaScript `Event` representing this `RunEvent`
	///
	/// # Panics
	/// This function panics if the JS `CustomEvent(type, options)` constructor
	/// fails
	#[must_use]
	#[cfg(target_arch = "wasm32")]
	pub fn into_js(&self) -> Event {
		let init = CustomEventInit::new();
		init.set_detail(&self.details());

		CustomEvent::new_with_event_init_dict(self.name(), &init)
			.expect("JS `new CustomEvent(...)` failed")
			.into()
	}
}

impl Display for RunEvent {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Loaded(game) => f.write_fmt(format_args!("`web-bg` loaded, starting '{game}'")),
			Self::Initialized(None) => f.write_str("`web-bg` initialized"),
			Self::Initialized(Some(d)) => {
				f.write_fmt(format_args!("`web-bg` initialized in {} ms", d.as_millis()))
			}
			Self::Started(None) => f.write_str("`web-bg` started"),
			Self::Started(Some(d)) => {
				f.write_fmt(format_args!("`web-bg` started in {} ms", d.as_millis()))
			}
			Self::Panicked(None) => f.write_str("`web-bg` panicked"),
			Self::Panicked(Some(d)) => f.write_fmt(format_args!("`web-bg` panicked:\n{d}")),
		}
	}
}

/// Initialize event time measurements
///
/// This should be called as early as possible during application startup
pub fn init() {
	STARTUP_TIME.get_or_init(Instant::now);
}

/// Dispatch the `Loaded` event once
///
/// This should be called as early as possible during application startup
///
/// # Panics
/// This function panics on `wasm32` if the `window` JS object or its
/// `dispatchEvent` method are unavailable or throw
pub fn loaded(game: &'static str) {
	static ONCE: Once = Once::new();

	ONCE.call_once(|| {
		let event = RunEvent::Loaded(game);

		#[cfg(target_arch = "wasm32")]
		web_sys::window()
			.expect("JS `window` not available")
			.dispatch_event(&event.into_js())
			.expect("JS `dispatchEvent` failed");

		info!("{event}");
	});
}

/// Dispatch the `Initialized` event once
///
/// This should be called after startup systems have run
///
/// # Panics
/// This function panics on `wasm32` if the `window` JS object or its
/// `dispatchEvent` method are unavailable or throw
pub fn initialized() {
	static ONCE: Once = Once::new();

	ONCE.call_once(|| {
		let dur = STARTUP_TIME.get().map(Instant::elapsed);
		let event = RunEvent::Initialized(dur);

		#[cfg(target_arch = "wasm32")]
		web_sys::window()
			.expect("JS `window` not available")
			.dispatch_event(&event.into_js())
			.expect("JS `dispatchEvent` failed");

		info!("{event}");
	});
}

/// Dispatch the `Started` event once
///
/// This should be called after update systems have run
///
/// This function skips its first call and dispatches the event the second time
/// it's called
///
/// # Panics
/// This function panics on `wasm32` if the `window` JS object or its
/// `dispatchEvent` method are unavailable or throw
pub fn started() {
	static SKIP: Once = Once::new();
	static ONCE: Once = Once::new();

	if SKIP.is_completed() {
		ONCE.call_once(|| {
			let dur = STARTUP_TIME.get().map(Instant::elapsed);
			let event = RunEvent::Started(dur);

			#[cfg(target_arch = "wasm32")]
			web_sys::window()
				.expect("JS `window` not available")
				.dispatch_event(&event.into_js())
				.expect("JS `dispatchEvent` failed");

			info!("{event}");
		});
	}

	SKIP.call_once(|| ());
}

/// Dispatch the `Panicked` event
///
/// This should be called from a panic handler or hook
///
/// # Panics
/// This function panics on `wasm32` if the `window` JS object or its
/// `dispatchEvent` method are unavailable or throw
pub fn panic(info: String) {
	let event = RunEvent::Panicked(Some(info));

	#[cfg(target_arch = "wasm32")]
	web_sys::window()
		.expect("JS `window` not available")
		.dispatch_event(&event.into_js())
		.expect("JS `dispatchEvent` failed");

	error!("{event}");
}

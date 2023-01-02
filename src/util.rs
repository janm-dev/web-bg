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

		const GAMES: &[(&str, $crate::util::SystemsFn, $crate::util::SystemsFn)] = &[
			$(
				#[cfg(feature = $feat)] ($feat, $name::startup_systems, $name::systems),
			)*
		];

		$(
			#[cfg(not(feature = $feat))]
		)*
		compile_error!("At least one minigame must be enabled");
	}
}

/// A pointer to a function that gives you systems
pub type SystemsFn = fn() -> SystemSet;

//! `web-bg` utilities and other miscellaneous things.

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

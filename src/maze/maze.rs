use std::{
	fmt::{Debug, Formatter, Result as FmtResult},
	ops::Neg,
};

use bevy::{prelude::*, window::PrimaryWindow};
use rand::{
	rngs::SmallRng,
	seq::{IteratorRandom, SliceRandom},
	Rng, SeedableRng,
};
#[cfg(feature = "profile")]
use tracing::instrument;

use self::Direction::{Bottom, Left, Right, Top};

pub const MAZE_SIZE: UVec2 = UVec2::splat(128);
pub const MAZE_ROOMS: usize = 128;

pub const TILE_SIZE: Vec2 = Vec2::new(32.0, 32.0);
pub const TILE_SCALE: f32 = 5.0;
pub const WALL_THICKNESS: f32 = 4.0;

pub const SUBTILE_SIZE: Vec2 = Vec2::new(16.0, 16.0);
pub const SUBTILE_SCALE: f32 = 2.0 / 5.0;

#[derive(Resource)]
pub struct Maze {
	pub width: u32,
	pub height: u32,
	pub tiles: Box<[Tile]>,
	pub wall: Handle<Image>,
	pub floor: [Handle<Image>; 2],
}

impl Maze {
	/// Create a new `Maze`
	///
	/// # Panic
	/// Panics if the maze is not `width * height` tiles large
	pub fn new(
		maze: impl Into<Box<[Tile]>>,
		width: u32,
		height: u32,
		wall: Handle<Image>,
		floor: [Handle<Image>; 2],
	) -> Self {
		let tiles = maze.into();

		assert_eq!(
			width * height,
			u32::try_from(tiles.len()).unwrap(),
			"the maze's size is incorrect"
		);

		Self {
			width,
			height,
			tiles,
			wall,
			floor,
		}
	}

	/// Get the index into `tiles` for `(x, y)`
	pub fn idx(&self, x: u32, y: u32) -> usize {
		usize::try_from(y * self.width + x).unwrap()
	}

	/// Get the tile at `(x, y)`
	///
	/// # Panic
	/// Panics if `x` is not less than the maze's width or `y` is not less than
	/// the maze's height
	pub fn get(&self, x: u32, y: u32) -> Tile {
		assert!(x < self.width, "x must be less than the maze's width");
		assert!(y < self.height, "y must be less than the maze's height");

		self.tiles[self.idx(x, y)]
	}

	/// Spawn the tile at `(x, y)` at the given location
	pub fn spawn_tile(
		&self,
		x: u32,
		y: u32,
		loc: Vec2,
		commands: &mut Commands,
		rng: &mut impl Rng,
	) {
		let tile = self.get(x, y);

		commands
			.spawn((tile, TilePos { x, y }, SpatialBundle {
				transform: Transform {
					translation: Vec3 {
						x: loc.x,
						y: loc.y,
						..default()
					},
					scale: Vec3::splat(TILE_SCALE),
					..default()
				},
				..default()
			}))
			.with_children(|builder| {
				let i = self.idx(x, y);
				let is_fully_closed = tile.is_closed(Top)
					&& tile.is_closed(Right)
					&& tile.is_closed(Bottom)
					&& tile.is_closed(Left);

				for sy in -2i8..=2 {
					for sx in -2i8..=2 {
						let texture = if is_fully_closed || is_edge(i, sx, sy, &self.tiles) {
							self.wall.clone()
						} else {
							self.floor
								.choose(rng)
								.expect("there are no floor textures")
								.clone()
						};

						builder.spawn(SpriteBundle {
							texture,
							transform: Transform {
								translation: Vec3 {
									x: SUBTILE_SIZE.x * SUBTILE_SCALE * f32::from(sx),
									y: SUBTILE_SIZE.y * SUBTILE_SCALE * f32::from(sy),
									z: -1.0,
								},
								scale: Vec3::splat(SUBTILE_SCALE),
								..default()
							},
							sprite: Sprite {
								// The sprites have a bit of padding due to issues with MSAA on
								// sprite edges. This padding is cut off here.
								rect: Some(Rect::new(1.0, 1.0, 16.0, 16.0)),
								custom_size: Some(Vec2::new(16.0, 16.0)),
								..default()
							},
							..default()
						});
					}
				}
			});
	}
}

impl Debug for Maze {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("Maze")
			.field("width", &self.width)
			.field("height", &self.height)
			.finish_non_exhaustive()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
	Top,
	Right,
	Bottom,
	Left,
}

impl Neg for Direction {
	type Output = Self;

	fn neg(self) -> Self::Output {
		match self {
			Self::Top => Self::Bottom,
			Self::Right => Self::Left,
			Self::Bottom => Self::Top,
			Self::Left => Self::Right,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct TilePos {
	pub x: u32,
	pub y: u32,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Tile(u8);

impl Tile {
	pub const fn closed() -> Self {
		Self(0b1111)
	}

	/// Open the given `side` of this Tile
	pub fn open(&mut self, side: Direction) -> &mut Self {
		match side {
			Direction::Top => self.0 &= 0b0111,
			Direction::Right => self.0 &= 0b1011,
			Direction::Bottom => self.0 &= 0b1101,
			Direction::Left => self.0 &= 0b1110,
		}

		self
	}

	/// Whether the given `side` of this Tile is open
	pub const fn is_open(self, side: Direction) -> bool {
		match side {
			Direction::Top => self.0 & 0b1000 == 0,
			Direction::Right => self.0 & 0b0100 == 0,
			Direction::Bottom => self.0 & 0b0010 == 0,
			Direction::Left => self.0 & 0b0001 == 0,
		}
	}

	/// Whether the given `side` of this Tile is closed
	pub const fn is_closed(self, side: Direction) -> bool {
		!self.is_open(side)
	}
}

impl Default for Tile {
	fn default() -> Self {
		Self::closed()
	}
}

#[cfg_attr(feature = "profile", instrument(skip_all))]
pub fn initialize(mut commands: Commands, asset_server: Res<AssetServer>) {
	let mut rng = SmallRng::from_entropy();

	let wall = asset_server.load("maze/cave-wall.png");
	let floor = [
		asset_server.load("maze/cave-floor-1.png"),
		asset_server.load("maze/cave-floor-2.png"),
	];

	let maze = gen_maze(&mut rng, wall, floor);

	commands.insert_resource(maze);
}

#[allow(clippy::cast_possible_truncation, clippy::type_complexity)]
#[cfg_attr(feature = "profile", instrument(skip_all))]
pub fn spawn_visible_tiles(
	mut commands: Commands,
	maze: Res<Maze>,
	tiles: Query<&TilePos, With<Tile>>,
	window: Query<&Window, (With<PrimaryWindow>, Without<Tile>, Without<Camera2d>)>,
	camera: Query<&Transform, (With<Camera2d>, Changed<Transform>, Without<Tile>)>,
) {
	#[allow(clippy::cast_precision_loss)]
	fn tile_position(i: u32) -> Vec2 {
		Vec2 {
			x: (i32::try_from(i % MAZE_SIZE.x).unwrap() - i32::try_from(MAZE_SIZE.x / 2).unwrap())
				as f32 * TILE_SCALE
				* TILE_SIZE.x,
			y: (i32::try_from(i / MAZE_SIZE.x).unwrap() - i32::try_from(MAZE_SIZE.y / 2).unwrap())
				as f32 * TILE_SCALE
				* TILE_SIZE.y,
		}
	}

	let Ok(window) = window.get_single() else {
		return;
	};

	let Ok(camera) = camera.get_single() else {
		return;
	};

	let existing_tiles = tiles.iter().copied().collect::<Vec<_>>();

	let new_tiles = maze
		.tiles
		.iter()
		.copied()
		.enumerate()
		.filter(|&(i, _)| {
			let Vec2 { x, y } = tile_position(i as u32);
			let width = TILE_SIZE.x.mul_add(TILE_SCALE, window.width());
			let height = TILE_SIZE.y.mul_add(TILE_SCALE, window.height());
			let x_extent =
				(camera.translation.x - width / 2.0)..(camera.translation.x + width / 2.0);
			let y_extent =
				(camera.translation.y - height / 2.0)..(camera.translation.y + height / 2.0);
			x_extent.contains(&x) && y_extent.contains(&y)
		})
		.filter_map(|(i, _)| {
			let pos = TilePos {
				x: i as u32 % maze.width,
				y: i as u32 / maze.width,
			};

			(!existing_tiles.contains(&pos)).then_some((pos.x, pos.y, i))
		});

	for (x, y, i) in new_tiles {
		maze.spawn_tile(
			x,
			y,
			tile_position(i as _),
			&mut commands,
			&mut rand::thread_rng(),
		);
	}
}

#[allow(clippy::type_complexity)]
#[cfg_attr(feature = "profile", instrument(skip_all))]
pub fn despawn_invisible_tiles(
	mut commands: Commands,
	tiles: Query<(Entity, &Transform), With<Tile>>,
	window: Query<&Window, (With<PrimaryWindow>, Without<Tile>, Without<Camera2d>)>,
	camera: Query<&Transform, (With<Camera2d>, Changed<Transform>, Without<Tile>)>,
) {
	let Ok(window) = window.get_single() else {
		return;
	};

	let Ok(camera) = camera.get_single() else {
		return;
	};

	let mut old_tiles = tiles.iter().filter(|&(_, t)| {
		let Vec3 { x, y, .. } = t.translation;
		let width = TILE_SIZE.x.mul_add(TILE_SCALE * 2.0, window.width());
		let height = TILE_SIZE.y.mul_add(TILE_SCALE * 2.0, window.height());
		let x_extent = (camera.translation.x - width / 2.0)..(camera.translation.x + width / 2.0);
		let y_extent = (camera.translation.y - height / 2.0)..(camera.translation.y + height / 2.0);
		!x_extent.contains(&x) || !y_extent.contains(&y)
	});

	if let Some((e, _)) = old_tiles.next() {
		// This is very slow, so only do one per frame
		commands.entity(e).despawn_recursive();
	}
}

/// Get the neighbors of a tile, along with the direction towards which they are
/// from the input tile position
fn neighbors(UVec2 { x, y }: UVec2) -> impl Iterator<Item = (UVec2, Direction)> {
	[
		((x, u32::min(y + 1, MAZE_SIZE.y - 1)), Top),
		((u32::min(x + 1, MAZE_SIZE.x - 1), y), Right),
		((x, y.saturating_sub(1)), Bottom),
		((x.saturating_sub(1), y), Left),
	]
	.into_iter()
	.map(|(v, d)| (UVec2::from(v), d))
}

/// Get the next tile in the maze for the usual recursive backtracking
/// algorithm
fn next_maze(pos: UVec2, visited: &[UVec2], mut rng: &mut impl Rng) -> Option<(UVec2, Direction)> {
	neighbors(pos)
		.filter(|(p, _)| !visited.contains(p))
		.choose(&mut rng)
}

/// Get the next tile in the maze for a modified version of the recursive
/// backtracking algorithm, generating a more cave-like environment, though it
/// may be a bit small
fn next_cave(pos: UVec2, visited: &[UVec2], rng: &mut impl Rng) -> Option<(UVec2, Direction)> {
	#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
	rng.gen_bool(0.999_f64.powi(visited.len() as i32))
		.then(|| next_maze(pos, visited, rng))
		.flatten()
}

/// The type of `next_maze` and `next_cave`, where `R` is a `rand::Rng`
type NextFn<R> = fn(UVec2, &[UVec2], &mut R) -> Option<(UVec2, Direction)>;

#[cfg_attr(feature = "profile", instrument(skip_all, fields(kind)))]
fn gen_maze<R: Rng>(mut rng: &mut R, wall: Handle<Image>, floor: [Handle<Image>; 2]) -> Maze {
	let us = |u32: u32| -> usize { u32.try_into().unwrap() };
	let idx = |UVec2 { x, y }| usize::try_from(y * MAZE_SIZE.x + x).unwrap();

	let mut maze = vec![Tile::default(); us(MAZE_SIZE.x) * us(MAZE_SIZE.y)];

	let mut pos = MAZE_SIZE / 2;
	let mut visited = vec![pos];
	let mut route = vec![pos];

	let (next, rooms): (NextFn<R>, usize) = if rng.gen_bool(0.75) {
		#[cfg(feature = "profile")]
		tracing::Span::current().record("kind", "maze");
		(next_maze, MAZE_ROOMS)
	} else {
		#[cfg(feature = "profile")]
		tracing::Span::current().record("kind", "cave");
		(next_cave, 0)
	};

	loop {
		let Some((next, dir)) = next(pos, &visited, rng) else {
			pos = if let Some(p) = route.pop() {
				p
			} else {
				break;
			};
			continue;
		};

		maze[idx(pos)].open(dir);
		maze[idx(next)].open(-dir);

		visited.push(next);
		route.push(next);

		pos = next;
	}

	for pos in (0..MAZE_SIZE.x)
		.flat_map(|x| (0..MAZE_SIZE.y).map(move |y| UVec2 { x, y }))
		.choose_multiple(&mut rng, rooms)
		.into_iter()
		.chain([MAZE_SIZE / 2])
	{
		maze[idx(pos)]
			.open(Direction::Top)
			.open(Direction::Right)
			.open(Direction::Bottom)
			.open(Direction::Left);

		for (pos, dir) in neighbors(pos) {
			maze[idx(pos)].open(-dir);
		}
	}

	Maze::new(maze, MAZE_SIZE.x, MAZE_SIZE.y, wall, floor)
}

fn is_edge(i: usize, x: i8, y: i8, maze: &[Tile]) -> bool {
	let maze_size = (
		usize::try_from(MAZE_SIZE.x).unwrap(),
		usize::try_from(MAZE_SIZE.y).unwrap(),
	);

	let tile = maze[i];
	let tile_is_edge = !(maze_size.0..=(maze_size.1 - 1) * maze_size.0).contains(&i)
		|| i % maze_size.0 == 0
		|| i % maze_size.0 == maze_size.0 - 1;

	match (x, y) {
		(-1..=1, -1..=1) => false,
		(-1..=1, 2) => tile.is_closed(Top),
		(2, -1..=1) => tile.is_closed(Right),
		(-1..=1, -2) => tile.is_closed(Bottom),
		(-2, -1..=1) => tile.is_closed(Left),
		(-2, 2) => {
			tile.is_closed(Top)
				|| tile.is_closed(Left)
				|| (!tile_is_edge
					&& (maze[i.saturating_sub(1)].is_closed(Top)
						|| maze[i.saturating_add(maze_size.0)].is_closed(Left)))
		}
		(2, 2) => {
			tile.is_closed(Top)
				|| tile.is_closed(Right)
				|| (!tile_is_edge
					&& (maze[i.saturating_add(1)].is_closed(Top)
						|| maze[i.saturating_add(maze_size.0)].is_closed(Right)))
		}
		(-2, -2) => {
			tile.is_closed(Bottom)
				|| tile.is_closed(Left)
				|| (!tile_is_edge
					&& (maze[i.saturating_sub(1)].is_closed(Bottom)
						|| maze[i.saturating_sub(maze_size.0)].is_closed(Left)))
		}
		(2, -2) => {
			tile.is_closed(Bottom)
				|| tile.is_closed(Right)
				|| (!tile_is_edge
					&& (maze[i.saturating_add(1)].is_closed(Bottom)
						|| maze[i.saturating_sub(maze_size.0)].is_closed(Right)))
		}
		_ => false,
	}
}

use std::ops::Neg;

use bevy::prelude::*;
use rand::{
	rngs::SmallRng,
	seq::{IteratorRandom, SliceRandom},
	Rng, SeedableRng,
};
#[cfg(feature = "profile")]
use tracing::instrument;

pub const MAZE_SIZE: UVec2 = UVec2::splat(128);
pub const MAZE_ROOMS: usize = 128;

pub const TILE_SIZE: Vec2 = Vec2::new(32.0, 32.0);
pub const TILE_SCALE: f32 = 5.0;
pub const WALL_THICKNESS: f32 = 4.0;

pub const SUBTILE_SIZE: Vec2 = Vec2::new(16.0, 16.0);
pub const SUBTILE_SCALE: f32 = 2.0 / 5.0;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileSide {
	Open,
	Closed,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Tile {
	pub top: TileSide,
	pub right: TileSide,
	pub bottom: TileSide,
	pub left: TileSide,
}

impl Tile {
	/// Open the given `side` of this Tile
	pub fn open(&mut self, side: Direction) -> &mut Self {
		match side {
			Direction::Top => self.top = TileSide::Open,
			Direction::Right => self.right = TileSide::Open,
			Direction::Bottom => self.bottom = TileSide::Open,
			Direction::Left => self.left = TileSide::Open,
		}

		self
	}

	/// Whether the given `side` of this Tile is open
	pub fn is_open(self, side: Direction) -> bool {
		match side {
			Direction::Top => self.top == TileSide::Open,
			Direction::Right => self.right == TileSide::Open,
			Direction::Bottom => self.bottom == TileSide::Open,
			Direction::Left => self.left == TileSide::Open,
		}
	}

	/// Whether the given `side` of this Tile is closed
	pub fn is_closed(self, side: Direction) -> bool {
		!self.is_open(side)
	}
}

impl Default for Tile {
	fn default() -> Self {
		Self {
			top: TileSide::Closed,
			right: TileSide::Closed,
			bottom: TileSide::Closed,
			left: TileSide::Closed,
		}
	}
}

/// Get the neighbors of a tile, along with the direction towards which they are
/// from the input tile position
fn neighbors(UVec2 { x, y }: UVec2) -> impl Iterator<Item = (UVec2, Direction)> {
	use self::Direction::{Bottom, Left, Right, Top};

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

#[cfg_attr(feature = "profile", instrument(skip(rng), fields(kind)))]
fn gen_maze<R: Rng>(mut rng: &mut R) -> Vec<Tile> {
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

	maze
}

use self::Direction::{Bottom, Left, Right, Top};

fn is_edge(i: usize, x: i8, y: i8, maze: &[Tile]) -> bool {
	let maze_size = (
		usize::try_from(MAZE_SIZE.x).expect("usize is probably at least 32 bits"),
		usize::try_from(MAZE_SIZE.y).expect("usize is probably at least 32 bits"),
	);

	let tile = maze[i];
	let tile_is_edge = !(maze_size.0..=(maze_size.1 - 1) * maze_size.0).contains(&i)
		|| i % maze_size.0 == 0
		|| i % maze_size.0 == maze_size.0 - 1;

	match (x, y) {
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

pub fn initialize(mut commands: Commands, asset_server: Res<AssetServer>) {
	let mut rng = SmallRng::from_entropy();
	let maze = gen_maze(&mut rng);

	let wall = asset_server.load("maze/cave-wall.png");
	let floor = [
		asset_server.load("maze/cave-floor-1.png"),
		asset_server.load("maze/cave-floor-2.png"),
	];

	#[allow(clippy::cast_precision_loss)]
	let tile_position = |i| Vec3 {
		x: (i32::try_from(i % MAZE_SIZE.x).unwrap() - i32::try_from(MAZE_SIZE.x / 2).unwrap())
			as f32 * TILE_SCALE
			* TILE_SIZE.x,
		y: (i32::try_from(i / MAZE_SIZE.x).unwrap() - i32::try_from(MAZE_SIZE.y / 2).unwrap())
			as f32 * TILE_SCALE
			* TILE_SIZE.y,
		z: -10.0,
	};

	for (i, tile) in maze
		.iter()
		.copied()
		.enumerate()
		.map(|(i, t)| (u32::try_from(i).unwrap(), t))
	{
		commands
			.spawn((tile, SpatialBundle {
				transform: Transform {
					translation: tile_position(i),
					scale: Vec3::splat(TILE_SCALE),
					..default()
				},
				..default()
			}))
			.with_children(|builder| {
				let is_fully_closed = tile.is_closed(Top)
					&& tile.is_closed(Right)
					&& tile.is_closed(Bottom)
					&& tile.is_closed(Left);

				for y in -2i8..=2 {
					for x in -2i8..=2 {
						let texture =
							if is_fully_closed || is_edge(i.try_into().unwrap(), x, y, &maze) {
								wall.clone()
							} else {
								floor
									.choose(&mut rng)
									.expect("there are no floor textures")
									.clone()
							};

						builder.spawn(SpriteBundle {
							texture,
							transform: Transform {
								translation: Vec3 {
									x: SUBTILE_SIZE.x * SUBTILE_SCALE * f32::from(x),
									y: SUBTILE_SIZE.y * SUBTILE_SCALE * f32::from(y),
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

use std::{
	f32::consts::PI,
	fmt::{Debug, Formatter, Result as FmtResult},
	ops::Neg,
};

use bevy::{prelude::*, window::PrimaryWindow};

use self::Direction::{Bottom, Left, Right, Top};
use crate::util::{Rand, TurboRand};

pub const MAZE_SIZE: UVec2 = UVec2::splat(128);
pub const MAZE_ROOMS: usize = 128;

pub const TILE_SIZE: Vec2 = Vec2::new(32.0, 32.0);
pub const TILE_SCALE: f32 = 5.0;
pub const WALL_THICKNESS: f32 = 4.0;

pub const SUBTILE_SIZE: Vec2 = Vec2::new(16.0, 16.0);
pub const SUBTILE_SCALE: f32 = 2.0 / 5.0;

#[derive(Resource)]
pub struct Maze {
	width: u32,
	height: u32,
	pub tiles: Box<[Tile]>,
	wall: Box<[Handle<StandardMaterial>]>,
	floor: Box<[Handle<StandardMaterial>]>,
	wall_mesh: Handle<Mesh>,
	floor_mesh: Handle<Mesh>,
	wall_material: Handle<StandardMaterial>,
}

impl Maze {
	/// Create a new `Maze`
	///
	/// # Panic
	/// Panics if the maze is not `width * height` tiles large
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		maze: impl Into<Box<[Tile]>>,
		width: u32,
		height: u32,
		wall: impl Into<Box<[Handle<StandardMaterial>]>>,
		floor: impl Into<Box<[Handle<StandardMaterial>]>>,
		wall_mesh: Handle<Mesh>,
		floor_mesh: Handle<Mesh>,
		wall_material: Handle<StandardMaterial>,
	) -> Self {
		let tiles = maze.into();
		let wall = wall.into();
		let floor = floor.into();

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
			wall_mesh,
			floor_mesh,
			wall_material,
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
	pub fn spawn_tile(&self, x: u32, y: u32, loc: Vec2, commands: &mut Commands, rng: &Rand) {
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

				let is_fully_open = tile.is_open(Top)
					&& tile.is_open(Right)
					&& tile.is_open(Bottom)
					&& tile.is_open(Left);
				let is_fully_closed = tile.is_closed(Top)
					&& tile.is_closed(Right)
					&& tile.is_closed(Bottom)
					&& tile.is_closed(Left);

				if !(is_fully_closed || is_fully_open) {
					self.spawn_tile_walls(builder, tile);
				}

				for sy in -2i8..=2 {
					for sx in -2i8..=2 {
						let texture = if is_fully_closed || is_edge(i, sx, sy, &self.tiles) {
							rng.sample(&self.wall)
								.expect("there are no wall textures")
								.clone()
						} else {
							rng.sample(&self.floor)
								.expect("there are no floor textures")
								.clone()
						};

						builder.spawn(PbrBundle {
							mesh: self.floor_mesh.clone(),
							material: texture.clone(),
							transform: Transform {
								translation: Vec3 {
									x: SUBTILE_SIZE.x * SUBTILE_SCALE * f32::from(sx),
									y: SUBTILE_SIZE.y * SUBTILE_SCALE * f32::from(sy),
									z: 0.0,
								},
								scale: Vec3::splat(SUBTILE_SCALE),
								..default()
							},
							..default()
						});
					}
				}
			});
	}

	fn spawn_tile_walls(&self, builder: &mut ChildBuilder, tile: Tile) {
		if tile.is_closed(Top) {
			builder.spawn(PbrBundle {
				mesh: self.wall_mesh.clone(),
				material: self.wall_material.clone(),
				transform: Transform {
					translation: Vec3 {
						x: 0.0,
						y: TILE_SIZE.y / 2.0,
						z: 0.0,
					},
					..default()
				},
				..default()
			});
		}

		if tile.is_closed(Bottom) {
			builder.spawn(PbrBundle {
				mesh: self.wall_mesh.clone(),
				material: self.wall_material.clone(),
				transform: Transform {
					translation: Vec3 {
						x: 0.0,
						y: -TILE_SIZE.y / 2.0,
						z: 0.0,
					},
					..default()
				},
				..default()
			});
		}

		if tile.is_closed(Right) {
			builder.spawn(PbrBundle {
				mesh: self.wall_mesh.clone(),
				material: self.wall_material.clone(),
				transform: Transform {
					translation: Vec3 {
						x: TILE_SIZE.x / 2.0,
						y: 0.0,
						z: 0.0,
					},
					rotation: Quat::from_rotation_z(PI / 2.0),
					..default()
				},
				..default()
			});
		}

		if tile.is_closed(Left) {
			builder.spawn(PbrBundle {
				mesh: self.wall_mesh.clone(),
				material: self.wall_material.clone(),
				transform: Transform {
					translation: Vec3 {
						x: -TILE_SIZE.x / 2.0,
						y: 0.0,
						z: 0.0,
					},
					rotation: Quat::from_rotation_z(PI / 2.0),
					..default()
				},
				..default()
			});
		}
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

#[allow(clippy::too_many_lines)]
#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn initialize(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	rng: Res<Rand>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let wall = asset_server.load("maze/cave-wall.png");
	let floor = [
		asset_server.load("maze/cave-floor-1.png"),
		asset_server.load("maze/cave-floor-2.png"),
	];

	let wall = [
		materials.add(StandardMaterial {
			base_color: Color::GRAY,
			base_color_texture: Some(wall.clone()),
			reflectance: 0.1,
			perceptual_roughness: 1.0,
			depth_map: Some(wall.clone()),
			emissive: Color::hsl(210.0, 0.3, 0.3),
			emissive_texture: Some(wall.clone()),
			normal_map_texture: Some(wall.clone()),
			unlit: false,
			..default()
		}),
		materials.add(StandardMaterial {
			base_color: Color::GRAY,
			base_color_texture: Some(wall.clone()),
			reflectance: 0.1,
			perceptual_roughness: 0.8,
			depth_map: Some(wall.clone()),
			emissive: Color::hsl(210.0, 0.3, 0.3),
			emissive_texture: Some(wall.clone()),
			normal_map_texture: Some(wall.clone()),
			unlit: false,
			..default()
		}),
		materials.add(StandardMaterial {
			base_color: Color::GRAY,
			base_color_texture: Some(wall.clone()),
			reflectance: 0.2,
			perceptual_roughness: 1.0,
			depth_map: Some(wall.clone()),
			emissive: Color::hsl(210.0, 0.3, 0.3),
			emissive_texture: Some(wall.clone()),
			normal_map_texture: Some(wall.clone()),
			unlit: false,
			..default()
		}),
		materials.add(StandardMaterial {
			base_color: Color::GRAY,
			base_color_texture: Some(wall.clone()),
			reflectance: 0.2,
			perceptual_roughness: 0.8,
			depth_map: Some(wall.clone()),
			emissive: Color::hsl(210.0, 0.3, 0.3),
			emissive_texture: Some(wall.clone()),
			normal_map_texture: Some(wall),
			unlit: false,
			..default()
		}),
	];

	let floor = [
		materials.add(StandardMaterial {
			base_color: Color::GRAY,
			base_color_texture: Some(floor[0].clone()),
			reflectance: 0.1,
			perceptual_roughness: 1.0,
			depth_map: Some(floor[0].clone()),
			emissive: Color::hsl(210.0, 0.3, 0.3),
			emissive_texture: Some(floor[0].clone()),
			normal_map_texture: Some(floor[0].clone()),
			unlit: false,
			..default()
		}),
		materials.add(StandardMaterial {
			base_color: Color::GRAY,
			base_color_texture: Some(floor[1].clone()),
			reflectance: 0.1,
			perceptual_roughness: 1.0,
			depth_map: Some(floor[1].clone()),
			emissive: Color::hsl(210.0, 0.3, 0.3),
			emissive_texture: Some(floor[1].clone()),
			normal_map_texture: Some(floor[1].clone()),
			unlit: false,
			..default()
		}),
		materials.add(StandardMaterial {
			base_color: Color::GRAY,
			base_color_texture: Some(floor[0].clone()),
			reflectance: 0.1,
			perceptual_roughness: 0.85,
			depth_map: Some(floor[0].clone()),
			emissive: Color::hsl(210.0, 0.3, 0.3),
			emissive_texture: Some(floor[0].clone()),
			normal_map_texture: Some(floor[0].clone()),
			unlit: false,
			..default()
		}),
		materials.add(StandardMaterial {
			base_color: Color::GRAY,
			base_color_texture: Some(floor[1].clone()),
			reflectance: 0.1,
			perceptual_roughness: 0.85,
			depth_map: Some(floor[1].clone()),
			emissive: Color::hsl(210.0, 0.3, 0.3),
			emissive_texture: Some(floor[1].clone()),
			normal_map_texture: Some(floor[1].clone()),
			unlit: false,
			..default()
		}),
	];

	let floor_mesh = meshes.add(Mesh::from(shape::Quad::new(SUBTILE_SIZE)));
	let wall_mesh = meshes.add(Mesh::from(shape::Box::new(
		SUBTILE_SIZE.x.mul_add(SUBTILE_SCALE, TILE_SIZE.x),
		SUBTILE_SIZE.y * SUBTILE_SCALE,
		10.0,
	)));

	let wall_material = materials.add(StandardMaterial {
		base_color: Color::rgba(1.0, 1.0, 1.0, 1.0),
		emissive: Color::rgba(0.0, 0.0, 0.0, 0.0),
		reflectance: 1.0,
		unlit: true,
		fog_enabled: false,
		..default()
	});

	let maze = gen_maze(&rng);
	let maze = Maze::new(
		maze,
		MAZE_SIZE.x,
		MAZE_SIZE.y,
		wall,
		floor,
		wall_mesh,
		floor_mesh,
		wall_material,
	);

	commands.insert_resource(maze);
}

#[allow(clippy::cast_possible_truncation, clippy::type_complexity)]
#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn spawn_visible_tiles(
	mut commands: Commands,
	maze: Res<Maze>,
	rng: Res<Rand>,
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
			let width = TILE_SIZE.x.mul_add(TILE_SCALE * 2.0, window.width());
			let height = TILE_SIZE.y.mul_add(TILE_SCALE * 2.0, window.height());
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
		maze.spawn_tile(x, y, tile_position(i as _), &mut commands, &rng);
	}
}

#[allow(clippy::type_complexity)]
#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
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
		let width = TILE_SIZE.x.mul_add(TILE_SCALE * 3.0, window.width());
		let height = TILE_SIZE.y.mul_add(TILE_SCALE * 3.0, window.height());
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
fn next_maze(pos: UVec2, visited: &[UVec2], rng: &Rand) -> Option<(UVec2, Direction)> {
	rng.sample_iter(neighbors(pos).filter(|(p, _)| !visited.contains(p)))
}

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
fn gen_maze(rng: &Rand) -> Vec<Tile> {
	let us = |u32: u32| -> usize { u32.try_into().unwrap() };
	let idx = |UVec2 { x, y }| usize::try_from(y * MAZE_SIZE.x + x).unwrap();

	let mut maze = vec![Tile::default(); us(MAZE_SIZE.x) * us(MAZE_SIZE.y)];

	let mut pos = MAZE_SIZE / 2;
	let mut visited = Vec::with_capacity(us(MAZE_SIZE.x) * us(MAZE_SIZE.y));
	visited.push(pos);
	let mut route = vec![pos];

	loop {
		let Some((next, dir)) = next_maze(pos, &visited, rng) else {
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

		#[cfg(feature = "debug")]
		#[allow(clippy::cast_precision_loss)]
		if visited.len() % 512 == 0 {
			debug!(
				"gen_maze - {:.2}%",
				100.0 * visited.len() as f32 / (MAZE_SIZE.x as f32 * MAZE_SIZE.y as f32)
			);
		}
	}

	for pos in rng
		.sample_multiple_iter(
			(0..MAZE_SIZE.x).flat_map(|x| (0..MAZE_SIZE.y).map(move |y| UVec2 { x, y })),
			MAZE_ROOMS,
		)
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

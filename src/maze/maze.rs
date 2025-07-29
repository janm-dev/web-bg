use std::{
	array,
	fmt::{Debug, Formatter, Result as FmtResult},
	ops::Neg,
};

use bevy::{
	color::palettes::css,
	prelude::*,
	render::render_resource::{
		Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
	},
	window::PrimaryWindow,
};
use bevy_light_2d::prelude::{LightOccluder2d, LightOccluder2dShape};
use image::{RgbaImage, imageops, load_from_memory};

use self::Direction::{Bottom, Left, Right, Top};
use crate::util::{Rand, TurboRand};

pub const MAZE_SIZE: UVec2 = UVec2::splat(128);
pub const MAZE_ROOMS: usize = 1024;

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
	textures: Box<[Handle<ColorMaterial>; 256]>,
	floor_mesh: Handle<Mesh>,
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
		textures: Box<[Handle<ColorMaterial>; 256]>,
		floor_mesh: Handle<Mesh>,
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
			textures,
			floor_mesh,
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
	#[allow(clippy::too_many_arguments)]
	pub fn spawn_tile(
		&self,
		x: u32,
		y: u32,
		loc: Vec2,
		commands: &mut Commands,
		asset_server: &AssetServer,
		texture_atlases: &mut Assets<TextureAtlasLayout>,
		rng: &Rand,
	) {
		let tile = self.get(x, y);

		let ti = tile_bits(self.idx(x, y), &self.tiles);

		commands
			.spawn((
				tile,
				TilePos { x, y },
				(
					Mesh2d(self.floor_mesh.clone()),
					MeshMaterial2d(self.textures[ti as usize].clone()),
					Transform {
						translation: Vec3 {
							x: loc.x,
							y: loc.y,
							..default()
						},
						scale: Vec3::splat(TILE_SCALE),
						..default()
					},
				),
			))
			.with_children(|builder| {
				let is_fully_open = tile.is_open(Top)
					&& tile.is_open(Right)
					&& tile.is_open(Bottom)
					&& tile.is_open(Left);
				let is_fully_closed = tile.is_closed(Top)
					&& tile.is_closed(Right)
					&& tile.is_closed(Bottom)
					&& tile.is_closed(Left);

				if !(is_fully_closed || is_fully_open) {
					Self::spawn_tile_walls(builder, tile);
				}

				if tile.has_food() {
					super::food::spawn(builder, asset_server, texture_atlases, rng);
				}
			});
	}

	fn spawn_tile_walls(builder: &mut ChildSpawnerCommands, tile: Tile) {
		if tile.is_closed(Top) {
			builder.spawn((
				LightOccluder2d {
					shape: LightOccluder2dShape::Rectangle {
						half_size: Vec2 {
							x: TILE_SIZE
								.x
								.mul_add(TILE_SCALE, SUBTILE_SIZE.x * SUBTILE_SCALE * TILE_SCALE),
							y: SUBTILE_SIZE.y * SUBTILE_SCALE * TILE_SCALE,
						} / 2.0,
					},
				},
				Transform {
					translation: Vec3 {
						x: 0.0,
						y: TILE_SIZE.y / 2.0,
						z: 0.0,
					},
					..default()
				},
			));
		}

		if tile.is_closed(Bottom) {
			builder.spawn((
				LightOccluder2d {
					shape: LightOccluder2dShape::Rectangle {
						half_size: Vec2 {
							x: TILE_SIZE
								.x
								.mul_add(TILE_SCALE, SUBTILE_SIZE.x * SUBTILE_SCALE * TILE_SCALE),
							y: SUBTILE_SIZE.y * SUBTILE_SCALE * TILE_SCALE,
						} / 2.0,
					},
				},
				Transform {
					translation: Vec3 {
						x: 0.0,
						y: -TILE_SIZE.y / 2.0,
						z: 0.0,
					},
					..default()
				},
			));
		}

		if tile.is_closed(Right) {
			builder.spawn((
				LightOccluder2d {
					shape: LightOccluder2dShape::Rectangle {
						half_size: Vec2 {
							x: SUBTILE_SIZE.x * SUBTILE_SCALE * TILE_SCALE,
							y: TILE_SIZE
								.y
								.mul_add(TILE_SCALE, SUBTILE_SIZE.y * SUBTILE_SCALE * TILE_SCALE),
						} / 2.0,
					},
				},
				Transform {
					translation: Vec3 {
						x: TILE_SIZE.x / 2.0,
						y: 0.0,
						z: 0.0,
					},
					..default()
				},
			));
		}

		if tile.is_closed(Left) {
			builder.spawn((
				LightOccluder2d {
					shape: LightOccluder2dShape::Rectangle {
						half_size: Vec2 {
							x: SUBTILE_SIZE.x * SUBTILE_SCALE * TILE_SCALE,
							y: TILE_SIZE
								.y
								.mul_add(TILE_SCALE, SUBTILE_SIZE.y * SUBTILE_SCALE * TILE_SCALE),
						} / 2.0,
					},
				},
				Transform {
					translation: Vec3 {
						x: -TILE_SIZE.x / 2.0,
						y: 0.0,
						z: 0.0,
					},
					..default()
				},
			));
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

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
fn gen_tile_textures(
	wall: &[&[u8]],
	floor: &[&[u8]],
	images: &mut Assets<Image>,
	rng: &Rand,
) -> [Handle<Image>; 256] {
	let mut res = array::from_fn::<_, 256, _>(|_| None);

	let wall = wall
		.iter()
		.map(|data| load_from_memory(data).expect("invalid image").into_rgba8())
		.collect::<Vec<_>>();

	let floor = floor
		.iter()
		.map(|data| load_from_memory(data).expect("invalid image").into_rgba8())
		.collect::<Vec<_>>();

	for bits in 0u8..=255u8 {
		let tile = Tile(bits & 0b1111);

		let is_edge = |sx, sy| match (sx, sy) {
			(1..=3, 0) => tile.is_closed(Top),
			(4, 1..=3) => tile.is_closed(Right),
			(1..=3, 4) => tile.is_closed(Bottom),
			(0, 1..=3) => tile.is_closed(Left),
			(0, 0) => tile.is_closed(Top) || tile.is_closed(Left) || (bits & 0b1000_0000 != 0),
			(4, 0) => tile.is_closed(Top) || tile.is_closed(Right) || (bits & 0b0100_0000 != 0),
			(0, 4) => tile.is_closed(Bottom) || tile.is_closed(Left) || (bits & 0b0010_0000 != 0),
			(4, 4) => tile.is_closed(Bottom) || tile.is_closed(Right) || (bits & 0b0001_0000 != 0),
			_ => false,
		};

		let is_fully_closed = tile.is_closed(Top)
			&& tile.is_closed(Right)
			&& tile.is_closed(Bottom)
			&& tile.is_closed(Left);

		let mut image = RgbaImage::from_raw(5 * 16, 5 * 16, vec![0; 4 * 5 * 16 * 5 * 16]).unwrap();

		for sy in 0..5 {
			for sx in 0..5 {
				let subimage = if is_fully_closed || is_edge(sx, sy) {
					rng.sample(&wall).expect("there are no wall images")
				} else {
					rng.sample(&floor).expect("there are no floor images")
				};

				imageops::overlay(&mut image, subimage, sx * 16, sy * 16);
			}
		}

		let handle = images.add(Image {
			data: Some(image.into_vec()),
			texture_descriptor: TextureDescriptor {
				label: None,
				size: Extent3d {
					width: 5 * 16,
					height: 5 * 16,
					..default()
				},
				dimension: TextureDimension::D2,
				format: TextureFormat::Rgba8UnormSrgb,
				mip_level_count: 1,
				sample_count: 1,
				usage: TextureUsages::TEXTURE_BINDING
					| TextureUsages::COPY_DST
					| TextureUsages::RENDER_ATTACHMENT,
				view_formats: &[],
			},
			texture_view_descriptor: None,
			..default()
		});
		res[bits as usize] = Some(handle);
	}

	res.map(|o| o.expect("image creation failed"))
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

	pub const fn set_food(&mut self, has_food: bool) -> &mut Self {
		if self.has_food() != has_food {
			self.0 ^= 0b0001_0000;
		}

		self
	}

	pub const fn has_food(self) -> bool {
		self.0 & 0b0001_0000 != 0
	}

	/// Open the given `side` of this Tile
	pub const fn open(&mut self, side: Direction) -> &mut Self {
		match side {
			Direction::Top => self.0 &= 0b1111_0111,
			Direction::Right => self.0 &= 0b1111_1011,
			Direction::Bottom => self.0 &= 0b1111_1101,
			Direction::Left => self.0 &= 0b1111_1110,
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

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn initialize(
	mut commands: Commands,
	rng: Res<Rand>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
	mut images: ResMut<Assets<Image>>,
) {
	let wall = [&include_bytes!("../../assets/maze/cave-wall.png")[..]];
	let floor = [
		&include_bytes!("../../assets/maze/cave-floor-1.png")[..],
		&include_bytes!("../../assets/maze/cave-floor-2.png")[..],
	];

	let floor_mesh = meshes.add(Rectangle::from_size(TILE_SIZE));

	let textures = gen_tile_textures(&wall, &floor, &mut images, &rng).map(|h| {
		materials.add(ColorMaterial {
			color: css::GRAY.into(),
			texture: Some(h),
			..default()
		})
	});

	let maze = gen_maze(&rng);

	let maze = Maze::new(
		maze,
		MAZE_SIZE.x,
		MAZE_SIZE.y,
		Box::new(textures),
		floor_mesh,
	);

	commands.insert_resource(maze);
}

#[allow(
	clippy::cast_possible_truncation,
	clippy::type_complexity,
	clippy::too_many_arguments
)]
#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn spawn_visible_tiles(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
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

	let Ok(window) = window.single() else {
		return;
	};

	let Ok(camera) = camera.single() else {
		return;
	};

	let existing_tiles = tiles.iter().copied().collect::<Vec<_>>();

	let new_tiles = (0..maze.tiles.len())
		.filter(|&i| {
			let Vec2 { x, y } = tile_position(i as u32);
			let width = TILE_SIZE.x.mul_add(TILE_SCALE * 2.0, window.width());
			let height = TILE_SIZE.y.mul_add(TILE_SCALE * 2.0, window.height());
			let x_extent =
				(camera.translation.x - width / 2.0)..(camera.translation.x + width / 2.0);
			let y_extent =
				(camera.translation.y - height / 2.0)..(camera.translation.y + height / 2.0);
			x_extent.contains(&x) && y_extent.contains(&y)
		})
		.filter_map(|i| {
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
			&asset_server,
			&mut texture_atlases,
			&rng,
		);
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
	let Ok(window) = window.single() else {
		return;
	};

	let Ok(camera) = camera.single() else {
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
		commands.entity(e).despawn();
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
			.open(Direction::Left)
			.set_food(true);

		for (pos, dir) in neighbors(pos) {
			maze[idx(pos)].open(-dir);
		}
	}

	maze
}

fn tile_bits(i: usize, maze: &[Tile]) -> u8 {
	let maze_size = (
		usize::try_from(MAZE_SIZE.x).unwrap(),
		usize::try_from(MAZE_SIZE.y).unwrap(),
	);

	let tile = maze[i];
	let tile_is_edge = !(maze_size.0..=(maze_size.1 - 1) * maze_size.0).contains(&i)
		|| i.is_multiple_of(maze_size.0)
		|| i % maze_size.0 == maze_size.0 - 1;

	let mut res = tile.0 & 0b1111;

	if !tile_is_edge {
		if maze[i.saturating_sub(1)].is_closed(Top)
			|| maze[i.saturating_add(maze_size.0)].is_closed(Left)
		{
			res |= 0b1000_0000;
		}

		if maze[i.saturating_add(1)].is_closed(Top)
			|| maze[i.saturating_add(maze_size.0)].is_closed(Right)
		{
			res |= 0b0100_0000;
		}

		if maze[i.saturating_sub(1)].is_closed(Bottom)
			|| maze[i.saturating_sub(maze_size.0)].is_closed(Left)
		{
			res |= 0b0010_0000;
		}

		if maze[i.saturating_add(1)].is_closed(Bottom)
			|| maze[i.saturating_sub(maze_size.0)].is_closed(Right)
		{
			res |= 0b0001_0000;
		}
	}

	res
}

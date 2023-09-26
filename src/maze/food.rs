use bevy::prelude::*;

use super::{maze::Tile, player::Player};
use crate::util::{Rand, TurboRand};

pub const FOOD_SIZE: Vec2 = Vec2::new(32.0, 32.0);
pub const FOOD_SCALE: f32 = 1.0 / 5.0;
pub const FOOD_AMOUNT: usize = 49;

pub const EATING_THRESHOLD: f32 = 1024.0;

#[derive(Debug, Component)]
pub struct Food;

pub fn spawn(
	builder: &mut ChildBuilder,
	asset_server: &AssetServer,
	texture_atlases: &mut Assets<TextureAtlas>,
	rng: &Rand,
) {
	let foods_handle = asset_server.load("maze/food.png");
	let foods_atlas = TextureAtlas::from_grid(foods_handle, FOOD_SIZE, 1, FOOD_AMOUNT, None, None);
	let foods_atlas_handle = texture_atlases.add(foods_atlas);

	let index = rng.usize(0..FOOD_AMOUNT);

	builder.spawn((Food, SpriteSheetBundle {
		texture_atlas: foods_atlas_handle,
		sprite: TextureAtlasSprite::new(index),
		transform: Transform {
			scale: Vec3::splat(FOOD_SCALE),
			..default()
		},
		..default()
	}));
}

pub fn eat(
	mut commands: Commands,
	player: Query<&GlobalTransform, (With<Player>, Without<Food>)>,
	food: Query<(&GlobalTransform, &Parent, Entity), With<Food>>,
	mut tiles: Query<&mut Tile>,
) {
	let player = player.single().translation();

	let mut current_food = None;
	for food in &food {
		if food.0.translation().distance_squared(player) < EATING_THRESHOLD {
			current_food = Some((food.1, food.2));
			break;
		}
	}

	if let Some((parent, entity)) = current_food {
		tiles
			.get_mut(parent.get())
			.expect("food's tile not found")
			.set_food(false);

		commands.entity(entity).despawn();
	}
}

pub fn dim(
	player: Query<&GlobalTransform, (With<Player>, Without<Food>)>,
	mut food: Query<(&GlobalTransform, &mut TextureAtlasSprite), With<Food>>,
) {
	let player = player.single().translation();

	for (trans, mut sprite) in &mut food {
		let d = trans.translation().distance_squared(player);

		sprite.color.set_a(10000.0 / d);
	}
}

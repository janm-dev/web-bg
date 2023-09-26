use std::fmt::{Display as FmtDisplay, Formatter, Result as FmtResult, Write};

use bevy::prelude::*;

use super::{maze::Tile, player::Player};
use crate::util::{Rand, TurboRand};

pub const FOOD_SIZE: Vec2 = Vec2::new(32.0, 32.0);
pub const FOOD_SCALE: f32 = 1.0 / 5.0;
pub const FOOD_AMOUNT: usize = 49;

pub const EATING_THRESHOLD: f32 = 1024.0;

#[derive(Debug, Component)]
pub struct Food;

#[derive(Debug, Default, Component)]
pub struct FoodEaten(u16);

impl FoodEaten {
	pub fn incr(&mut self) {
		self.0 = self.0.saturating_add(1);
	}
}

impl FmtDisplay for FoodEaten {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let val = self.0.saturating_sub(1);

		if val > 99 {
			f.write_char('+')
		} else {
			f.write_fmt(format_args!("{val}"))
		}
	}
}

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

pub fn init_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
	let plate = asset_server.load("maze/plate.png");

	commands
		.spawn(ImageBundle {
			background_color: Color::WHITE.into(),
			style: Style {
				position_type: PositionType::Absolute,
				bottom: Val::Percent(5.0),
				right: Val::Percent(5.0),
				width: Val::Px(128.0),
				height: Val::Px(128.0),
				display: Display::Flex,
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				..default()
			},
			image: UiImage {
				texture: plate,
				..default()
			},
			..default()
		})
		.with_children(|builder| {
			builder.spawn((
				TextBundle::from_section("0", TextStyle {
					font: asset_server.load("fonts/pixel.ttf"),
					font_size: 64.0,
					color: Color::BLACK,
				})
				.with_text_alignment(TextAlignment::Center)
				.with_style(Style {
					position_type: PositionType::Relative,
					..default()
				}),
				FoodEaten::default(),
			));
		});
}

pub fn update_ui(mut counter: Query<(&FoodEaten, &mut Text), Changed<FoodEaten>>) {
	if let Ok((counter, mut text)) = counter.get_single_mut() {
		text.sections[0].value = counter.to_string();
	};
}

pub fn eat(
	mut commands: Commands,
	player: Query<&GlobalTransform, (With<Player>, Without<Food>)>,
	food: Query<(&GlobalTransform, &Parent, Entity), With<Food>>,
	mut counter: Query<&mut FoodEaten>,
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
		counter.single_mut().incr();
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

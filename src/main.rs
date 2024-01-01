use bevy::prelude::*;

mod fixp;
mod phys;

use crate::fixp::*;
use crate::phys::*;

#[derive(Component)]
struct Player {
	velocity: PhysVec
}

#[derive(Component, Default)]
struct FrameInput {
	direction: Vec2,
	jump_pressed: bool,
	grab_pressed: bool,
	dash_pressed: bool
}

#[derive(Component)]
struct BasicPlayerInput {}

fn basic_inputs(
	mut query: Query<&mut FrameInput, With<BasicPlayerInput>>,
	keys: Res<Input<KeyCode>>
) {
	let mut dir: Vec2 = Vec2::ZERO;

	if keys.pressed(KeyCode::A) {
		dir.x -= 1.0;
	}
	if keys.pressed(KeyCode::D) {
		dir.x += 1.0;
	}
	if keys.pressed(KeyCode::W) {
		dir.y += 1.0;
	}
	if keys.pressed(KeyCode::S) {
		dir.y -= 1.0;
	}

	dir = dir.normalize_or_zero();
	let jump_pressed = keys.pressed(KeyCode::N);
	let grab_pressed = keys.pressed(KeyCode::M);
	let dash_pressed = keys.pressed(KeyCode::Comma);

	for mut fi in query.iter_mut() {
		fi.direction = dir;
		fi.jump_pressed = jump_pressed;
		fi.grab_pressed = grab_pressed;
		fi.dash_pressed = dash_pressed;
	}
}

fn setup_camera(mut commands: Commands) {
	commands.spawn(Camera2dBundle::default());
}

fn setup_player(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
	commands.spawn((
		Player { velocity: phys::zero() },
		BasicPlayerInput {},
		FrameInput { ..Default::default() },
		SolidColorPhysAABBBundle::new(
			aabb_tiles(0, 0, 1, 1),
			Color::rgb(1.0, 1.0, 0.5),
			&mut meshes, &mut materials
		)
	));

	commands.spawn(
		SolidColorPhysAABBBundle::new(aabb_tiles(0, -3, 5, 1),
		Color::rgb(0.2, 0.2, 0.2),
		&mut meshes, &mut materials)
	);
}

fn player_update(mut query: Query<(&mut Player, &FrameInput)>) {
	for (mut player, input) in query.iter_mut() {
		let dist_x = (input.direction.x * 256.0 * 1.5) as i32;
		let dist_y = (input.direction.y * 256.0 * 1.5) as i32;
		player.velocity.x = FixP(dist_x);
		player.velocity.y = FixP(dist_y);
	}
}

fn player_physics(world: &mut World) {
	// Collect player ids for performing physics
	let player_ids: Vec<Entity> = world.query::<(Entity, With<Player>)>()
		.iter(&world)
		.map(|x| x.0)
		.collect();

	for id in player_ids {
		// Step one: fetch original AABB for computing physics
		let mut aabb = world.get::<PhysAABB>(id).unwrap().clone();
		let velocity = world.get::<Player>(id).unwrap().velocity;

		// Step two: perform a move_and_slide
		phys::move_and_slide(&mut aabb, id, velocity, world);

		// Step three: store AABB back into world
		*world.get_mut::<PhysAABB>(id).unwrap() = aabb;
	}
}

/// System that takes a physics object's AABB and computes a visual Transform
/// associated with that.
fn render_aabb_to_transform(mut query: Query<(&mut Transform, &PhysAABB)>) {
	for (mut tform, aabb) in query.iter_mut() {
		tform.translation = Vec3::new(
			f32::from(aabb.pos.x),
			f32::from(aabb.pos.y),
			tform.translation.z
		);
	}
}

fn main() {
    App::new()
		.add_plugins(DefaultPlugins)
		.add_systems(Startup, (setup_camera, setup_player))
		// Input systems: convert inputs into FrameInputs 
		.add_systems(Update, basic_inputs)
		// Render systems: convert game state into renderable state
		.add_systems(Update, render_aabb_to_transform)
		// Physics systems: must run at a fixed rate
		.add_systems(FixedUpdate, player_update)
		.add_systems(FixedUpdate, player_physics)
		.add_systems(Update, bevy::window::close_on_esc)
		.run();
}

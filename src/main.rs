use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

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

#[derive(Resource)]
struct PhysLerpAccumulator {
	accumulator: f32
}

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
	let mut cam = Camera2dBundle::default();
	cam.projection.scaling_mode = ScalingMode::FixedVertical(16.0 * 20.0);
	commands.spawn(cam);
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
			aabb_tiles(0, 0, 1, 2),
			Color::rgb(1.0, 1.0, 0.5),
			&mut meshes, &mut materials
		)
	));
	

	commands.spawn(
		SolidColorPhysAABBBundle::new(aabb_tiles(0, -3, 5, 1),
		Color::rgb(0.2, 0.2, 0.2),
		&mut meshes, &mut materials)
	);

	commands.spawn(
		SolidColorPhysAABBBundle::new(aabb_tiles(4, 0, 2, 6),
		Color::rgb(0.2, 0.2, 0.2),
		&mut meshes, &mut materials)
	);

	commands.spawn((
		SolidColorPhysAABBBundle::new(aabb_tiles(2, 0, 1, 1),
		Color::rgb(1.0, 0.9, 0.8),
		&mut meshes, &mut materials
		), Pushable{})
	);
}

fn physics_frame_start(mut query: Query<(&mut PhysLerpPos, &PhysAABB)>, mut acc: ResMut<PhysLerpAccumulator>, time: Res<Time<Fixed>>) {
	for (mut lerppos, aabb) in query.iter_mut() {
		lerppos.pos = aabb.pos;
	}

	acc.accumulator -= time.delta_seconds();
	//dbg!(acc.accumulator);
	//dbg!(time.delta_seconds());
}

fn player_update(mut query: Query<(&mut Player, &FrameInput)>) {
	// subpx / sec^2
	const ACCEL: i32 = 256 * 16 * 16;
	// subpx / sec
	const MAX_VEL: i32 = 256 * 16 * 4;

	for (mut player, input) in query.iter_mut() {
		let input = PhysVec {
			x: (input.direction.x * 256.0) as i32,
			y: (input.direction.y * 256.0) as i32
		};
		
		let accel = input * ACCEL;
		player.velocity += (accel / PHYS_FPS);
		player.velocity.clamp_length(MAX_VEL);
	}
}

fn player_physics(world: &mut World) {
	// Collect player ids for performing physics
	let player_ids: Vec<Entity> = world.query::<(Entity, With<Player>)>()
		.iter(&world)
		.map(|x| x.0)
		.collect();

	for id in player_ids {
		let mut player = &world.get_mut::<Player>(id).unwrap();

		phys::move_and_slide(id,  player.velocity, world);
	}

	
}



/// System that takes a physics object's AABB and computes a visual Transform
/// associated with that.
fn render_aabb_to_transform(mut query: Query<(&mut Transform, &PhysAABB, &PhysLerpPos)>, mut acc: ResMut<PhysLerpAccumulator>, time: Res<Time>) {
	acc.accumulator += time.delta_seconds();
	let lerp = acc.accumulator / PHYS_TIMESTEP;
	let lerp = f32::clamp(lerp, 0.0, 1.0);
	
	for (mut tform, aabb, lpos) in query.iter_mut() {
		let pos0 = Vec2::new(fixp_to_f32(lpos.pos.x), fixp_to_f32(lpos.pos.y));
		let pos1 = Vec2::new(fixp_to_f32(aabb.pos.x), fixp_to_f32(aabb.pos.y));

		let pos = pos0.lerp(pos1, lerp);

		tform.translation = Vec3::new(
			pos.x + 0.5 * fixp_to_f32(aabb.size.x),
			pos.y + 0.5 * fixp_to_f32(aabb.size.y),
			tform.translation.z
		);
	}
}

fn main() {
    App::new()
		.add_plugins(DefaultPlugins)
		.insert_resource(PhysLerpAccumulator{ accumulator: 0.0 })
		.add_systems(Startup, (setup_camera, setup_player))
		// Input systems: convert inputs into FrameInputs 
		.add_systems(Update, basic_inputs)
		// Render systems: convert game state into renderable state
		.add_systems(Update, render_aabb_to_transform)
		// Physics systems: must run at a fixed rate
		.add_systems(FixedUpdate, physics_frame_start)
		.add_systems(FixedUpdate, player_update.after(physics_frame_start))
		.add_systems(FixedUpdate, player_physics.after(player_update))
		.add_systems(Update, bevy::window::close_on_esc)
		.run();
}

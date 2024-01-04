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

#[derive(Component)]
struct BasicMovingPlatform {
	length: i32,
	length_remaining: i32,
	speed: i32,
	direction: i32
}

fn moving_platform_physics(world: &mut World) {
	// Collect player ids for performing physics
	let mp_ids: Vec<Entity> = world.query::<(Entity, With<BasicMovingPlatform>)>()
		.iter(&world)
		.map(|x| x.0)
		.collect();

	for id in mp_ids {
		let mp = world.get::<BasicMovingPlatform>(id).unwrap();

		let increment = i32::min(mp.length_remaining * PHYS_FPS, mp.speed);

		let vel = PhysVec { x: increment * mp.direction, y: 0 };
		//dbg!(vel);

		let res = phys::move_and_slide(id, vel, world);

		let mut mp = world.get_mut::<BasicMovingPlatform>(id).unwrap();

		mp.length_remaining -= i32::abs(res.x);
		if mp.length_remaining <= 0 {
			mp.length_remaining = mp.length;
			mp.direction *= -1;
		}

		//let v = phys::move_and_slide(id,  player.velocity, world);
	}

	
}

fn setup_player(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
	commands.spawn((
		Pushable {},
		Player { velocity: phys::zero() },
		BasicPlayerInput {},
		FrameInput { ..Default::default() },
		SolidColorPhysAABBBundle::new(
			aabb_tiles(0, 0, 1, 2),
			Color::rgb(1.0, 1.0, 0.5),
			&mut meshes, &mut materials
		)
	));

	commands.spawn((
		SolidColorPhysAABBBundle::new(
			aabb_tiles(-2, 0, 3, 1),
			Color::rgb(0.6, 0.6, 0.6),
			&mut meshes, &mut materials
		),
		BasicMovingPlatform { length: 256 * 16 * 400, length_remaining: 256 * 16 * 400, speed: 256 * 16 * 8, direction: 1 }
	));
	

	commands.spawn(
		SolidColorPhysAABBBundle::new(aabb_tiles(0, -3, 11, 1),
		Color::rgb(0.4, 0.4, 0.4),
		&mut meshes, &mut materials)
	);

	commands.spawn(
		SolidColorPhysAABBBundle::new(aabb_tiles(4, 0, 1, 6),
		Color::rgb(0.4, 0.4, 0.4),
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
	const ACCEL: i32 = 256 * 16 * 64;
	// subpx / sec
	const MAX_VEL: i32 = 256 * 16 * 16;
	const GRAVITY: i32 = 256 * 16 * 32;

	for (mut player, finput) in query.iter_mut() {
		let input = PhysVec {
			x: (finput.direction.x * 256.0) as i32,
			y: (finput.direction.y * 256.0) as i32
		};
		
		let accel = input * ACCEL;
		player.velocity.x += (accel.x / PHYS_FPS);
		player.velocity.x = i32::clamp(player.velocity.x, -MAX_VEL, MAX_VEL);

		player.velocity.y -= GRAVITY / PHYS_FPS;
		if player.velocity.y < -MAX_VEL {
			player.velocity.y = -MAX_VEL;
		}

		if finput.jump_pressed {
			player.velocity.y = 256 * 16 * 16;
		}
		//player.velocity.clamp_length(MAX_VEL);

		player.velocity *= 255;
	}
}

fn player_physics(world: &mut World) {
	// Collect player ids for performing physics
	let player_ids: Vec<Entity> = world.query::<(Entity, With<Player>)>()
		.iter(&world)
		.map(|x| x.0)
		.collect();

	for id in player_ids {
		let player = world.get::<Player>(id).unwrap();

		let v = phys::move_and_slide(id,  player.velocity, world);

		let mut player = world.get_mut::<Player>(id).unwrap();
		player.velocity = v;
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
		.add_systems(FixedUpdate, moving_platform_physics.after(player_physics))
		.add_systems(Update, bevy::window::close_on_esc)
		// Make sure to always synchronize the FixedUpdate with our actual physics FPS
		.insert_resource(Time::<Fixed>::from_seconds(PHYS_TIMESTEP as f64))
		.run();
}

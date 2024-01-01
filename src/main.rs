use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

#[derive(Component)]
struct Player {}

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
		Player {},
		BasicPlayerInput {},
		FrameInput { ..Default::default() },
		MaterialMesh2dBundle {
			mesh: meshes.add(shape::Quad::new(Vec2::new(16.0, 16.0)).into()).into(),
			material: materials.add(ColorMaterial::from(Color::rgb(1.0, 1.0, 0.2))).into(),
			..Default::default()
		}
	));
}

fn player_physics(mut query: Query<(&mut Transform, &FrameInput), With<Player>>) {
	for (mut tform, input) in query.iter_mut() {
		tform.translation += input.direction.extend(0.0);
	}
}

fn render_player(mut query: Query<&mut Transform, With<Player>>) {
	for mut tform in query.iter_mut() {
		tform.rotate_z(13.0);
	}
}

fn main() {
    App::new()
		.add_plugins(DefaultPlugins)
		.add_systems(Startup, (setup_camera, setup_player))
		// Input systems: convert inputs into FrameInputs 
		.add_systems(Update, (basic_inputs))
		// Render systems: convert game state into renderable state
		.add_systems(Update, (render_player))
		// Physics systems: must run at a fixed rate
		.add_systems(FixedUpdate, player_physics)
		.add_systems(Update, bevy::window::close_on_esc)
		.run();
}

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

#[derive(Clone, Copy)]
struct FixP(i32);

impl From<FixP> for f32 {
    fn from(mut value: FixP) -> Self {
		if value.0 == 0 { return 0.0; }

		let sign = if value.0 < 0 {
			// TODO: handle MIN_INT
			value.0 = -value.0;
			true
		} else { false };

		// If not zero, we have a leading one. Find that leading one.
		let mut exponent: i32 = 31;
		let mut mantissa_mask: u32 = 0b01111111111111111111111100000000;
		let mut mantissa_shift: i32 = 8;
		while value.0 & (1 << exponent) == 0 {
			exponent -= 1;
			mantissa_mask >>= 1;
			mantissa_shift -= 1;
		}

		// Now we can extract the mantissa.
		let mut mantissa: u32 = (value.0 as u32) & mantissa_mask;
		if mantissa_shift >= 0 {
			mantissa >>= mantissa_shift;
		}
		else {
			mantissa <<= -mantissa_shift;
		}

		// At this point, the mantissa does not have the leading one, and is
		// positioned at the end of the number -- so we just have to insert
		// the exponent field and the sign field.

		// Note that we do not have to handle denormalized numbers, because they
		// have such a small exponent that none of our fixed point values
		// correspond to it.

		let bias: i32 = 127;
		let fixed_point: i32 = 8;
		let sign_bit: u32 = if sign { 1 } else { 0 };

		let bits: u32 = 
			mantissa |
			(((exponent + bias - fixed_point) as u32) << 23) |
			(sign_bit << 31);

		return f32::from_bits(bits);
    }
}

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
	let fix = FixP(0b1001000000);
	println!("test: {}", Into::<f32>::into(fix));

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

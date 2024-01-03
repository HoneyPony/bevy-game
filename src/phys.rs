use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

pub const PHYS_TIMESTEP: f32 = 1.0/120.0;
pub const PHYS_FPS: i32 = 120;

use crate::fixp::*;

#[derive(Component)]
pub struct Pushable {}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct PhysVec {
	pub x: fixp,
	pub y: fixp
}

impl core::ops::Add for PhysVec {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        PhysVec { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl core::ops::AddAssign for PhysVec {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl core::ops::Mul<fixp> for PhysVec {
	type Output = Self;

	fn mul(self, rhs: fixp) -> Self::Output {
		PhysVec { x: fix_mul(self.x, rhs), y: fix_mul(self.y, rhs) }
	}
}

impl core::ops::Div<i32> for PhysVec {
	type Output = Self;
	
	fn div(self, rhs: i32) -> Self::Output {
		PhysVec { x: self.x / rhs, y: self.y / rhs }
	}
}

impl core::ops::MulAssign<fixp> for PhysVec {
    fn mul_assign(&mut self, rhs: fixp) {
		*self = *self * rhs;
    }
}

impl PhysVec {
	pub fn clamp_length(&mut self, len: fixp) {
		let mut x64 = self.x as i64;
		let mut y64 = self.y as i64;
		let test = x64 * x64 + y64 * y64;
		let len = len as i64;
		if test > (len * len) {
			x64 *= len;
			y64 *= len;

			let sqrt = f64::sqrt(test as f64);
			let sqrt = sqrt as i64;

			if sqrt != 0 {
				x64 /= sqrt;
				y64 /= sqrt;

				self.x = x64 as i32;
				self.y = y64 as i32;
			}
		}
	}
}

pub fn zero() -> PhysVec {
	PhysVec { x: 0, y: 0 } 
}

#[derive(Component, Clone)]
pub struct PhysAABB {
	pub pos: PhysVec,
	pub size: PhysVec,
}

#[derive(Component)]
pub struct PhysLerpPos {
	pub pos: PhysVec
}

pub fn vec(x: i32, y: i32) -> PhysVec {
	return PhysVec{ x, y };
}

impl PhysAABB {
	pub fn bottom(&self) -> fixp { self.pos.y }
	pub fn top(&self) -> fixp { self.pos.y + self.size.y - 1 }

	pub fn left(&self) -> fixp { self.pos.x }
	pub fn right(&self) -> fixp { self.pos.x + self.size.x - 1 }
}

#[derive(Bundle)]
pub struct SolidColorPhysAABBBundle {
	pub aabb: PhysAABB,
	pub lerp: PhysLerpPos,
	pub mesh: MaterialMesh2dBundle<ColorMaterial>
}

impl SolidColorPhysAABBBundle {
	pub fn new(aabb: PhysAABB, color: Color, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<ColorMaterial>>) -> Self {
		let size_x = fixp_to_f32(aabb.size.x);
		let size_y = fixp_to_f32(aabb.size.y);

		let pos_x: f32 = fixp_to_f32(aabb.pos.x);
		let pos_y: f32 = fixp_to_f32(aabb.pos.y);
		let transform = Transform { translation: Vec3::new(pos_x, pos_y, 0.0), ..Default::default() };
		
		let lerp = PhysLerpPos { pos: aabb.pos };

		return SolidColorPhysAABBBundle {
			aabb,
			lerp,
			mesh: MaterialMesh2dBundle {
				mesh: meshes.add(shape::Quad::new(Vec2::new(size_x, size_y)).into()).into(),
				material: materials.add(ColorMaterial::from(color)).into(),
				transform,
				..Default::default()
			}
		}
	}
}

pub fn aabb_subpx(x: i32, y: i32, width: i32, height: i32) -> PhysAABB {
	return PhysAABB {
		pos: PhysVec { x, y },
		size: PhysVec { x: width, y: height }
	}
}

pub fn aabb_tiles(x: i32, y: i32, width: i32, height: i32) -> PhysAABB {
	let mul = 256 * 16;
	return aabb_subpx(x * mul, y * mul, width * mul, height * mul)
}

fn y_dist(r1: &PhysAABB, r2: &PhysAABB) -> Option<fixp> {
	if r1.top() < r2.bottom() {
		return Some( (r2.bottom() - r1.top()))
	}

	if r2.top() < r1.bottom() {
		return Some(- (r1.bottom() - r2.top()))
	}

	// Overlapping -- TODO figure out what we want to do
	None
}

fn x_dist(r1: &PhysAABB, r2: &PhysAABB) -> Option<fixp> {
	if r1.right() < r2.left() {
		return Some(( (r2.left() - r1.right())))
	}

	if r2.right() < r1.left() {
		return Some((- (r1.left() - r2.right())))
	}

	None
}

pub fn move_and_slide(entity: Entity, velocity: PhysVec, world: &mut World) -> PhysVec {
	// For each AABB in the world that isn't our own_id, we will clamp the velocity.
	let mut vx = velocity.x / PHYS_FPS;
	let mut vy = velocity.y / PHYS_FPS;

	let mut aabb = world.get::<PhysAABB>(entity).unwrap().clone();

	let vec: Vec<_> = world.query::<(Entity, With<PhysAABB>)>().iter(&world).collect();

	let mut ret_velocity = velocity;
	
	for (id, _) in vec {
		if id == entity { continue; }

		let mut vx_new = vx;
		let mut vy_new = vy;

		let mut other = world.get::<PhysAABB>(id).unwrap().clone();

		let mut pushed = false;
		loop {
			if let Some(x) = x_dist(&aabb, &other) {
				if i32::signum(x) == i32::signum(vx) {
					// If we're moving in this direction, clamp x if necessary.
					if i32::abs(vx) > i32::abs(x) - 1 {
						let x_ac = (i32::abs(x)) * i32::signum(x);

						let y = (vy * x_ac) / vx; // Fixed point multiply
						let mut test = aabb.clone();
						test.pos.y += y;
						if let None = y_dist(&test, &other) {
							// Now we know this is a collision.

							if !pushed && world.get::<Pushable>(id).is_some() {
								let mut v = velocity.clone();
								v.x -= x;
								v.y -= y;
								move_and_slide(id, v, world);
								other = world.get::<PhysAABB>(id).unwrap().clone();
								pushed = true;
								continue;
							}

							//vy = y;
							vx_new = i32::signum(vx) * (i32::abs(x) - 1);
							ret_velocity.x = 0; // Cancel out x velocity on x collision
						}
					}
				}
			}

			if let Some(y) = y_dist(&aabb, &other) {
				if i32::signum(y) == i32::signum(vy) {
					// If we're moving in this direction, clamp x if necessary.
					if i32::abs(vy) > i32::abs(y) - 1 {
						// Here, we know the y is potentially problematic.
						// What we have to do is project the AABB downwards to
						// see if it actually overlaps the X afterwards.

						let y_ac = (i32::abs(y)) * i32::signum(y);
						// Note: y is guaranteed not to be 0.
						let x = (vx * y_ac) / (vy); // Fixed point multiply
						let mut test = aabb.clone();
						test.pos.x += x;
						if let None = x_dist(&test, &other) {
							// Now we know this is a collision.
							if !pushed && world.get::<Pushable>(id).is_some() {
								let mut v = velocity.clone();
								v.x -= x;
								v.y -= y;
								move_and_slide(id, v, world);
								other = world.get::<PhysAABB>(id).unwrap().clone();
								pushed = true;
								continue;
							}

							//vx = x;
							vy_new = i32::signum(vy) * (i32::abs(y) - 1);
							ret_velocity.y = 0; // Cancel y velocity on y collision
						}
					}
				}
				//else if y == 0 {
				//	vy = 0;
				//}
			}
			break;
		}

		vx = vx_new;
		vy = vy_new;

		//println!("x dist: {} y dist: {}", x, y);
	}

	// Now apply the velocity to the AABB.
	aabb.pos.x += vx;
	aabb.pos.y += vy;
	*world.get_mut::<PhysAABB>(entity).unwrap() = aabb;

	ret_velocity
}

#[cfg(test)]
mod tests {
	use bevy::prelude::*;
	use super::*;

    #[test]
    fn basic_y() {
        let mut app = App::new();

		// Fake player
		let player = app.world.spawn(aabb_tiles(0, 0, 1, 1)).id();

		// Add obstacle
		app.world.spawn(aabb_tiles(0, -2, 5, 1));

		move_and_slide(player, vec(0, -256 * 16 * 8), &mut app.world);

		assert_eq!(app.world.get::<PhysAABB>(player).unwrap().pos, vec(256 * 16 * 0, 256 * 16 * -1));
    }

	#[test]
	fn corner() {
		let mut app = App::new();

		// Fake player
		let player = app.world.spawn(aabb_tiles(0, 0, 1, 1)).id();

		// Add obstacle
		app.world.spawn(aabb_tiles(2, -2, 1, 1));

		move_and_slide(player, vec(256 * 16 * 8, -256 * 16 * 8), &mut app.world);

		assert_eq!(app.world.get::<PhysAABB>(player).unwrap().pos, vec(256 * 16 * 1, 256 * 16 * -1));
	}
}
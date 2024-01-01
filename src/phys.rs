use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::fixp::*;

#[derive(Component, Clone, Copy)]
pub struct PhysVec {
	pub x: FixP,
	pub y: FixP
}

pub fn zero() -> PhysVec {
	PhysVec { x: FixP(0), y: FixP(0) } 
}

#[derive(Component, Clone)]
pub struct PhysAABB {
	pub pos: PhysVec,
	pub size: PhysVec
}

impl PhysAABB {
	pub fn top(&self) -> FixP { self.pos.y }
	pub fn bottom(&self) -> FixP { FixP(self.pos.y.0 + self.size.y.0) }

	pub fn left(&self) -> FixP { self.pos.x }
	pub fn right(&self) -> FixP { FixP(self.pos.x.0 + self.size.x.0) }
}

#[derive(Bundle)]
pub struct SolidColorPhysAABBBundle {
	pub aabb: PhysAABB,
	pub mesh: MaterialMesh2dBundle<ColorMaterial>
}

impl SolidColorPhysAABBBundle {
	pub fn new(aabb: PhysAABB, color: Color, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<ColorMaterial>>) -> Self {
		let size_x = f32::from(aabb.size.x);
		let size_y = f32::from(aabb.size.y);

		let pos_x: f32 = f32::from(aabb.pos.x);
		let pos_y: f32 = f32::from(aabb.pos.y);
		let transform = Transform { translation: Vec3::new(pos_x, pos_y, 0.0), ..Default::default() };
		
		return SolidColorPhysAABBBundle {
			aabb,
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
		pos: PhysVec { x: FixP(x), y: FixP(y) },
		size: PhysVec { x: FixP(width), y: FixP(height) }
	}
}

pub fn aabb_tiles(x: i32, y: i32, width: i32, height: i32) -> PhysAABB {
	let mul = 256 * 16;
	return aabb_subpx(x * mul, y * mul, width * mul, height * mul)
}

fn y_dist(r1: &PhysAABB, r2: &PhysAABB) -> FixP {
	if r1.top().0 < r2.bottom().0 {
		return FixP(- (r2.bottom().0 - r1.top().0))
	}

	if r2.top().0 < r1.bottom().0 {
		return FixP(r1.bottom().0 - r2.top().0)
	}

	// Overlapping -- TODO figure out what we want to do
	return FixP(0);
}

fn x_dist(r1: &PhysAABB, r2: &PhysAABB) -> FixP {
	if r1.right().0 < r2.left().0 {
		return FixP(- (r2.left().0 - r1.right().0))
	}

	if r2.right().0 < r1.left().0 {
		return FixP(r1.left().0 - r2.right().0)
	}

	return FixP(0)
}

pub fn move_and_slide(aabb: &mut PhysAABB, own_id: Entity, velocity: PhysVec, world: &mut World) -> PhysVec {
	// For each AABB in the world that isn't our own_id, we will clamp the velocity.
	let mut vx = velocity.x.0;
	let mut vy = velocity.y.0;
	
	for (other, id) in world.query::<(&PhysAABB, Entity)>().iter(&world) {
		if id == own_id { continue; }

		let x = x_dist(aabb, other).0;
		if i32::signum(x) == i32::signum(vx) {
			// If we're moving in this direction, clamp x if necessary.
			if i32::abs(vx) > i32::abs(x) {
				// If vx is larger than x, we need to scale the y velocity
				// by the amount that we're scaling vx by.
				if x != 0 {
					vy = (vy * vx) / (x * 256); // Fixed point multiply
				}

				vx = i32::signum(vx) * i32::abs(x);
			}
		}

		let y = y_dist(aabb, other).0;
		if i32::signum(y) == i32::signum(vy) {
			// If we're moving in this direction, clamp x if necessary.
			if i32::abs(vy) > i32::abs(y) {
				// If vx is larger than x, we need to scale the y velocity
				// by the amount that we're scaling vx by.
				if y != 0 {
					vx = (vy * vx) / (y * 256); // Fixed point multiply
				}

				vy = i32::signum(vy) * i32::abs(y);
			}
		}
	}

	// Now apply the velocity to the AABB.
	aabb.pos.x.0 += vx;
	aabb.pos.y.0 += vy;

	zero()
}
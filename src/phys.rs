use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::fixp::*;

#[derive(Component)]
pub struct Pushable {}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
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

pub fn vec(x: i32, y: i32) -> PhysVec {
	return PhysVec{ x: FixP(x), y: FixP(y) };
}

impl PhysAABB {
	pub fn bottom(&self) -> FixP { FixP(self.pos.y.0) }
	pub fn top(&self) -> FixP { FixP(self.pos.y.0 + self.size.y.0 - 1) }

	pub fn left(&self) -> FixP { self.pos.x }
	pub fn right(&self) -> FixP { FixP(self.pos.x.0 + self.size.x.0 - 1) }
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

fn y_dist(r1: &PhysAABB, r2: &PhysAABB) -> Option<FixP> {
	if r1.top().0 < r2.bottom().0 {
		return Some(FixP( (r2.bottom().0 - r1.top().0)))
	}

	if r2.top().0 < r1.bottom().0 {
		return Some(FixP(- (r1.bottom().0 - r2.top().0)))
	}

	// Overlapping -- TODO figure out what we want to do
	None
}

fn x_dist(r1: &PhysAABB, r2: &PhysAABB) -> Option<FixP> {
	if r1.right().0 < r2.left().0 {
		return Some(FixP( (r2.left().0 - r1.right().0)))
	}

	if r2.right().0 < r1.left().0 {
		return Some(FixP(- (r1.left().0 - r2.right().0)))
	}

	None
}

pub fn move_and_slide(entity: Entity, velocity: PhysVec, world: &mut World) -> PhysVec {
	// For each AABB in the world that isn't our own_id, we will clamp the velocity.
	let mut vx = velocity.x.0;
	let mut vy = velocity.y.0;

	let mut aabb = world.get::<PhysAABB>(entity).unwrap().clone();

	let vec: Vec<_> = world.query::<(Entity, With<PhysAABB>)>().iter(&world).collect();
	
	for (id, _) in vec {
		if id == entity { continue; }

		let mut vx_new = vx;
		let mut vy_new = vy;

		let mut other = world.get::<PhysAABB>(id).unwrap().clone();

		let mut pushed = false;
		loop {
			if let Some(x) = x_dist(&aabb, &other) {
				let x = x.0;
				if i32::signum(x) == i32::signum(vx) {
					// If we're moving in this direction, clamp x if necessary.
					if i32::abs(vx) > i32::abs(x) - 1 {
						let x_ac = (i32::abs(x)) * i32::signum(x);

						let y = (vy * x_ac) / vx; // Fixed point multiply
						let mut test = aabb.clone();
						test.pos.y.0 += y;
						if let None = y_dist(&test, &other) {
							// Now we know this is a collision.

							if !pushed && world.get::<Pushable>(id).is_some() {
								//let mx = x_dist(&aabb, &other).unwrap_or(FixP(0));
								//let my = y_dist(&aabb, &other).unwrap_or(FixP(0));
								let mut v = velocity.clone();
								v.x.0 -= x;
								v.y.0 -= y;
								move_and_slide(id, v, world);
								other = world.get::<PhysAABB>(id).unwrap().clone();
								pushed = true;
								continue;
							}

							//vy = y;
							vx_new = i32::signum(vx) * (i32::abs(x) - 1);
						}
					}
				}
			}

			if let Some(y) = y_dist(&aabb, &other) {
				let y = y.0;
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
						test.pos.x.0 += x;
						if let None = x_dist(&test, &other) {
							// Now we know this is a collision.
							if !pushed && world.get::<Pushable>(id).is_some() {
								//let mx = x_dist(&aabb, &other).unwrap_or(FixP(0));
								//let my = y_dist(&aabb, &other).unwrap_or(FixP(0));
								let mut v = velocity.clone();
								v.x.0 -= x;
								v.y.0 -= y;
								move_and_slide(id, v, world);
								other = world.get::<PhysAABB>(id).unwrap().clone();
								pushed = true;
								continue;
							}

							//vx = x;
							vy_new = i32::signum(vy) * (i32::abs(y) - 1);
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
	aabb.pos.x.0 += vx;
	aabb.pos.y.0 += vy;
	*world.get_mut::<PhysAABB>(entity).unwrap() = aabb;

	zero()
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
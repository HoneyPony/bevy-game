use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::fixp::*;

#[derive(Component)]
pub struct PhysVec {
	pub x: FixP,
	pub y: FixP
}

#[derive(Component)]
pub struct PhysAABB {
	pub pos: PhysVec,
	pub size: PhysVec
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

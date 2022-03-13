use crate::types::{Vector4, Quat};
use nvflex_sys::{NvFlexCollisionGeometry, NvFlexCollisionShapeType, NvFlexCapsuleGeometry, eNvFlexShapeSphere, NvFlexSphereGeometry};

#[derive(Debug)]
pub struct Sphere {
	pub pos: Vector4,
	pub rot: Quat,

	pub radius: f32
}

impl Sphere {
	pub fn new(pos: Vector4, rot: Quat, radius: f32) -> Self {
		Self {
			pos,
			rot,
			radius
		}
	}

	pub fn as_union(&self) -> NvFlexCollisionGeometry {
		NvFlexCollisionGeometry {
			sphere: {
				NvFlexSphereGeometry {
					radius: self.radius
				}
			}
		}
	}
}

impl From<Sphere> for super::Shape {
	fn from(sphere: Sphere) -> Self {
		super::Shape::Sphere(sphere)
	}
}
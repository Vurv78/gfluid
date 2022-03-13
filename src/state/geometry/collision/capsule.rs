use crate::types::{Vector4, Quat};
use nvflex_sys::{NvFlexCollisionGeometry, NvFlexCollisionShapeType, NvFlexCapsuleGeometry, eNvFlexShapeCapsule};

#[derive(Debug)]
pub struct Capsule {
	pub pos: Vector4,
	pub rot: Quat,

	pub radius: f32,
	pub half_height: f32,
}

impl Capsule {
	pub fn new(pos: Vector4, rot: Quat, radius: f32, half_height: f32) -> Self {
		Self {
			pos,
			rot,
			radius,
			half_height
		}
	}

	pub fn as_union(&self) -> NvFlexCollisionGeometry {
		NvFlexCollisionGeometry {
			capsule: {
				NvFlexCapsuleGeometry {
					radius: self.radius,
					halfHeight: self.half_height
				}
			}
		}
	}
}

impl From<Capsule> for super::Shape {
	fn from(capsule: Capsule) -> Self {
		super::Shape::Capsule(capsule)
	}
}
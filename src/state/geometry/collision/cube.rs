use crate::types::{Vector4, Quat};
use nvflex_sys::{NvFlexCollisionGeometry, NvFlexBoxGeometry, NvFlexCollisionShapeType, eNvFlexShapeBox};

#[derive(Debug)]
pub struct Cube {
	pub pos: Vector4,
	pub rot: Quat,
	pub extents: [f32; 3]
}

impl Cube {
	pub fn new(pos: Vector4, rot: Quat, extents: [f32; 3]) -> Self {
		Self {
			pos,
			rot,
			extents
		}
	}

	pub fn as_union(&self) -> NvFlexCollisionGeometry {
		NvFlexCollisionGeometry {
			box_: {
				NvFlexBoxGeometry {
					halfExtents: self.extents
				}
			}
		}
	}
}

impl From<Cube> for super::Shape {
	fn from(cube: Cube) -> Self {
		super::Shape::Cube(cube)
	}
}
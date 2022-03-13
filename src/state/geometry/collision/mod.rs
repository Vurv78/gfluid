use nvflex_sys::*;
use std::mem::size_of;

use crate::{
	config,
	types::{Quat, Vector3, Vector4}, helper::NvFlexMakeShapeFlags,
};

use crate::{FlexState, state::CreateError};

pub mod cube;
pub mod capsule;
pub mod sphere;

pub use cube::Cube;
pub use capsule::Capsule;
pub use sphere::Sphere;

#[derive(Debug)]
pub enum Shape {
	Cube(Cube),
	Capsule(Capsule),
	Sphere(Sphere),
}

impl Shape {
	pub fn as_union(&self) -> NvFlexCollisionGeometry {
		match self {
			Shape::Cube(cube) => cube.as_union(),
			Shape::Capsule(capsule) => capsule.as_union(),
			Shape::Sphere(sphere) => sphere.as_union(),
		}
	}

	pub fn kind(&self) -> i32 {
		match self {
			Shape::Cube(_) => eNvFlexShapeBox,
			Shape::Capsule(_) => eNvFlexShapeCapsule,
			Shape::Sphere(_) => eNvFlexShapeSphere,
		}
	}

	pub fn get_pos(&self) -> &Vector4 {
		match self {
			Shape::Cube(cube) => &cube.pos,
			Shape::Capsule(capsule) => &capsule.pos,
			Shape::Sphere(sphere) => &sphere.pos,
		}
	}

	pub fn get_rot(&self) -> &Quat {
		match self {
			Shape::Cube(cube) => &cube.rot,
			Shape::Capsule(capsule) => &capsule.rot,
			Shape::Sphere(sphere) => &sphere.rot,
		}
	}
}

// kind: NvFlexCollisionShapeType, pos: Vector4, rot: Quat

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct ShapeState {
	#[derivative(Debug = "ignore")]
	shapes: Vec<Shape>,
	has_changes: bool,

	pub buffer: *mut NvFlexBuffer,
	pub positions: *mut NvFlexBuffer,          // Vec<Vec4>
	pub rotations: *mut NvFlexBuffer,          // Vec<Quat>
	pub previous_positions: *mut NvFlexBuffer, // Vec<Vec4>
	pub previous_rotations: *mut NvFlexBuffer, // Vec<Quat>
	pub flags: *mut NvFlexBuffer,              // Vec<i32>
}

impl ShapeState {
	/// Allocates buffers used by the geometry state
	/// # Safety
	/// Do not call this function more than once
	pub unsafe fn new(flex: *mut NvFlexLibrary, max: usize) -> Self {
		Self {
			has_changes: false,

			shapes: Vec::with_capacity(max),

			buffer: NvFlexAllocBuffer(
				flex,
				max as i32,
				size_of::<NvFlexCollisionGeometry>() as i32,
				eNvFlexBufferHost,
			),

			positions: NvFlexAllocBuffer(flex, max as i32, size_of::<Vector4>() as i32, eNvFlexBufferHost),

			rotations: NvFlexAllocBuffer(flex, max as i32, size_of::<Quat>() as i32, eNvFlexBufferHost),

			previous_positions: NvFlexAllocBuffer(
				flex,
				max as i32,
				size_of::<Vector4>() as i32,
				eNvFlexBufferHost,
			),

			previous_rotations: NvFlexAllocBuffer(
				flex,
				max as i32,
				size_of::<Quat>() as i32,
				eNvFlexBufferHost,
			),

			flags: NvFlexAllocBuffer(flex, max as i32, size_of::<i32>() as i32, eNvFlexBufferHost),
		}
	}

	pub fn get_count(&self) -> usize {
		self.shapes.len()
	}

	pub fn get_list(&self) -> &Vec<Shape> {
		&self.shapes
	}

	pub fn register(&mut self, shape: Shape) -> Result<(), CreateError> {
		let count = self.get_count();

		if count >= self.shapes.capacity() {
			return Err( CreateError::Max );
		}

		unsafe {
			let geometry = NvFlexMap(self.buffer, eNvFlexMapWait) as *mut NvFlexCollisionGeometry;
			let positions = NvFlexMap(self.positions, eNvFlexMapWait) as *mut Vector4;
			let rotations = NvFlexMap(self.rotations, eNvFlexMapWait) as *mut Quat;
			let previous_positions =
				NvFlexMap(self.previous_positions, eNvFlexMapWait) as *mut Vector4;
			let previous_rotations =
				NvFlexMap(self.previous_rotations, eNvFlexMapWait) as *mut Quat;

			let flags = NvFlexMap(self.flags, eNvFlexMapWait) as *mut i32;
			let flag = NvFlexMakeShapeFlags(shape.kind(), false);

			geometry.add(count).write(shape.as_union());
			positions.add(count).write(*shape.get_pos());
			rotations.add(count).write(*shape.get_rot());

			previous_positions.add(count).write(*shape.get_pos());
			previous_rotations.add(count).write(*shape.get_rot());

			flags.add(count).write(flag);

			self.unmap();
		}

		self.shapes.push(shape);

		self.has_changes = true;

		Ok(())
	}

	pub fn unmap(&self) {
		unsafe {
			NvFlexUnmap(self.buffer);
			NvFlexUnmap(self.positions);
			NvFlexUnmap(self.rotations);
			NvFlexUnmap(self.previous_positions);
			NvFlexUnmap(self.previous_rotations);
			NvFlexUnmap(self.flags);
		}
	}

	/// Pushes shape changes to the FleX state
	/// # Safety
	/// This is safe, assuming you don't manually map the buffers
	pub fn flush(&mut self, solver: *mut NvFlexSolver) {
		if !self.has_changes {
			return;
		}

		unsafe {
			NvFlexSetShapes(
				solver,
				self.buffer,
				self.positions,
				self.rotations,
				self.previous_positions,
				self.previous_rotations,
				self.flags,
				self.get_count() as i32,
			);
		}

		self.has_changes = false;
	}
}

impl Drop for ShapeState {
	fn drop(&mut self) {
		unsafe {
			NvFlexFreeBuffer(self.buffer);
			NvFlexFreeBuffer(self.positions);
			NvFlexFreeBuffer(self.rotations);
			NvFlexFreeBuffer(self.previous_positions);
			NvFlexFreeBuffer(self.previous_rotations);
			NvFlexFreeBuffer(self.flags);
		}
	}
}

use nvflex_sys::*;
use std::mem::size_of;

use crate::{
	config,
	types::{Quat, Vector3, Vector4},
};

use crate::FlexState;

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct ShapeState {
	#[derivative(Debug = "ignore")]
	shapes: Vec<NvFlexCollisionGeometry>,

	max: i32, // Todo: Remove this and just create Vec::with_capacity(max)
	count: i32,
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
	pub unsafe fn new(flex: *mut NvFlexLibrary, max: i32) -> Self {
		Self {
			has_changes: false,

			shapes: vec![],

			max,
			count: 0,

			buffer: NvFlexAllocBuffer(
				flex,
				max,
				size_of::<NvFlexCollisionGeometry>() as i32,
				eNvFlexBufferHost,
			),

			positions: NvFlexAllocBuffer(flex, max, size_of::<Vector4>() as i32, eNvFlexBufferHost),

			rotations: NvFlexAllocBuffer(flex, max, size_of::<Quat>() as i32, eNvFlexBufferHost),

			previous_positions: NvFlexAllocBuffer(
				flex,
				max,
				size_of::<Vector4>() as i32,
				eNvFlexBufferHost,
			),

			previous_rotations: NvFlexAllocBuffer(
				flex,
				max,
				size_of::<Quat>() as i32,
				eNvFlexBufferHost,
			),

			flags: NvFlexAllocBuffer(flex, max, size_of::<i32>() as i32, eNvFlexBufferHost),
		}
	}

	pub fn get_count(&self) -> i32 {
		self.count
	}

	pub fn create(&mut self, shape: NvFlexCollisionGeometry, pos: Vector4, rot: Quat, flag: i32) {
		self.shapes.push(shape);

		let count = self.count as isize;

		unsafe {
			let geometry = NvFlexMap(self.buffer, eNvFlexMapWait) as *mut NvFlexCollisionGeometry;
			let positions = NvFlexMap(self.positions, eNvFlexMapWait) as *mut Vector4;
			let rotations = NvFlexMap(self.rotations, eNvFlexMapWait) as *mut Quat;
			let previous_positions =
				NvFlexMap(self.previous_positions, eNvFlexMapWait) as *mut Vector4;
			let previous_rotations =
				NvFlexMap(self.previous_rotations, eNvFlexMapWait) as *mut Quat;
			let flags = NvFlexMap(self.flags, eNvFlexMapWait) as *mut i32;

			geometry.offset(count).write(shape);

			positions.offset(count).write(pos);

			rotations.offset(count).write(rot);

			previous_positions.offset(count).write(pos);

			previous_rotations.offset(count).write(rot);

			flags.offset(count).write(flag);

			self.unmap();
		}

		self.count += 1;
		self.has_changes = true;
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
	/// Make sure that all of the buffers have been unmapped before calling this.
	pub unsafe fn flush(&mut self, solver: *mut NvFlexSolver) {
		if !self.has_changes {
			return;
		}

		NvFlexSetShapes(
			solver,
			self.buffer,
			self.positions,
			self.rotations,
			self.previous_positions,
			self.previous_rotations,
			self.flags,
			self.count,
		);

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

use nvflex_sys::*;
use std::mem::size_of;

use crate::{
	config,
	types::{Quat, Vector3, Vector4},
};

use crate::FlexState;

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct TriangleState {
	max: i32,
	count: i32,
	has_changes: bool,

	pub buffer: *mut NvFlexBuffer,  // Vec<i32>
	pub normals: *mut NvFlexBuffer, // Vec<Vector3>
	pub uvs: *mut NvFlexBuffer,     // Vec<Vector3>
}

impl TriangleState {
	/// Allocates buffers used by the geometry state
	/// # Safety
	/// Do not call this function more than once
	pub fn new(flex: *mut NvFlexLibrary, max: i32) -> Self {
		unsafe {
			Self {
				max,
				count: 0,
				has_changes: false,

				buffer: NvFlexAllocBuffer(flex, max, size_of::<i32>() as i32, eNvFlexBufferHost),
				normals: NvFlexAllocBuffer(flex, max, size_of::<Vector3>() as i32, eNvFlexBufferHost),
				uvs: NvFlexAllocBuffer(flex, max, size_of::<Vector3>() as i32, eNvFlexBufferHost),
			}
		}
	}

	pub fn get_count(&self) -> i32 {
		self.count
	}

	pub fn unmap(&self) {
		unsafe {
			NvFlexUnmap(self.buffer);
			NvFlexUnmap(self.normals);
			NvFlexUnmap(self.uvs);
		}
	}

	/// Pushes shape changes to the FleX state
	/// # Safety
	/// Same as [ShapeState]
	pub fn flush(&mut self, solver: *mut NvFlexSolver) {
		if !self.has_changes {
			return;
		}

		unsafe {
			NvFlexSetDynamicTriangles(solver, self.buffer, self.normals, self.count);
		}

		self.has_changes = false;
	}
}

impl Drop for TriangleState {
	fn drop(&mut self) {
		unsafe {
			NvFlexFreeBuffer(self.buffer);
			NvFlexFreeBuffer(self.normals);
			NvFlexFreeBuffer(self.uvs);
		}
	}
}

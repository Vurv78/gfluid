use crate::types::*;

#[derive(Debug)]
pub struct ParticleFactory {
	pub nparticles: usize,

	/// Return values from NvFlexMap(...)
	buffer: *mut Vector4,
	velocities: *mut Vector3,
	phases: *mut i32,
	active_indices: *mut i32,
}

impl ParticleFactory {
	pub fn new(
		offset: Option<usize>,
		buffer: *mut Vector4,
		velocities: *mut Vector3,
		phases: *mut i32,
		indices: *mut i32,
	) -> Self {
		Self {
			nparticles: offset.unwrap_or(0),

			buffer,
			velocities,
			phases,
			active_indices: indices,
		}
	}

	pub fn create(&mut self, pos: Vector4, velocity: Vector3, phase: i32, _active: bool) {
		let index = self.nparticles;

		unsafe {
			self.buffer.add(index).write(pos);
			self.velocities.add(index).write(velocity);
			self.phases.add(index).write(phase);

			// Assumes particle is active for now.
			self.active_indices.add(index).write(index as i32);
		}
		self.nparticles += 1;
	}
}

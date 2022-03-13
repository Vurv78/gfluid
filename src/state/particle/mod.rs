use nvflex_sys::*;

use crate::{config, types::*};
use std::mem::size_of;

mod factory;

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct ParticleState {
	has_changes: bool,

	// (Index, Active)
	particles: Vec<(usize, bool)>,

	pub buffer: *mut NvFlexBuffer,
	pub velocities: *mut NvFlexBuffer,
	pub phases: *mut NvFlexBuffer,
	pub active_indices: *mut NvFlexBuffer,
}

impl ParticleState {
	/// # Safety
	/// Do not call this function more than once
	pub unsafe fn new(flex: *mut NvFlexLibrary, max: usize) -> Self {
		Self {
			has_changes: false,

			particles: Vec::with_capacity(max),
			// active: vec![],

			buffer: NvFlexAllocBuffer(
				flex,
				max as i32,
				size_of::<Vector4>() as i32,
				eNvFlexBufferHost,
			),

			velocities: NvFlexAllocBuffer(
				flex,
				max as i32,
				size_of::<Vector3>() as i32,
				eNvFlexBufferHost,
			),

			phases: NvFlexAllocBuffer(
				flex,
				max as i32,
				size_of::<i32>() as i32,
				eNvFlexBufferHost,
			),

			active_indices: NvFlexAllocBuffer(
				flex,
				max as i32,
				size_of::<i32>() as i32,
				eNvFlexBufferHost,
			),
		}
	}

	pub fn get_count(&self) -> usize {
		self.particles.len()
	}

	pub fn get_active_count(&self) -> usize {
		// self.particles.iter().filter(|(_, active)| *active).count()
		self.particles.len()
	}

	/// Adds a particle to FleX
	/// Note the changes won't be applied to flex immediately, you need to call [self.flush]
	/// Also this is very inefficient since it maps and unmaps every call..
	pub fn create(&mut self, pos: Vector4, vel: Vector3, phase: i32, active: bool) {
		let count = self.get_count();
		unsafe {
			let particles = NvFlexMap(self.buffer, eNvFlexMapWait) as *mut Vector4;
			let velocities = NvFlexMap(self.velocities, eNvFlexMapWait) as *mut Vector3;
			let phases = NvFlexMap(self.phases, eNvFlexMapWait) as *mut i32;
			let active_indices = NvFlexMap(self.active_indices, eNvFlexMapWait) as *mut i32;

			particles
				.add(count)
				.write(Vector4(50.0 * count as f32, 0.0, 5000.0, 2.0));

			velocities.add(count).write(Vector3(0.0, 0.0, -5.0));
			phases.add(count).write(phase);

			// Assume active for now.
			active_indices.add(count).write(count as i32);

			self.unmap();
		}

		self.particles.push( (count, active) );
		self.has_changes = true;
	}

	pub fn unmap(&self) {
		unsafe {
			NvFlexUnmap(self.buffer);
			NvFlexUnmap(self.velocities);
			NvFlexUnmap(self.phases);
			NvFlexUnmap(self.active_indices);
		}
	}

	pub unsafe fn get(&self, solver: *mut NvFlexSolver) -> Option<Vec<Particle>> {
		NvFlexGetParticles(solver, self.buffer, std::ptr::null());
		NvFlexGetVelocities(solver, self.velocities, std::ptr::null());
		NvFlexGetPhases(solver, self.velocities, std::ptr::null());

		let particles = NvFlexMap(self.buffer, eNvFlexMapWait) as *mut Vector4;
		let velocities = NvFlexMap(self.velocities, eNvFlexMapWait) as *mut Vector3;
		let phases = NvFlexMap(self.phases, eNvFlexMapWait) as *mut i32;

		let mut pvec = vec![];
		for i in 0 .. self.get_count() {
			let particle = particles.add(i);
			if particle.is_null() {
				break;
			}

			let velocity = velocities.add(i);
			let phase = phases.add(i);

			pvec.push(Particle {
				pdata: particle.as_ref()?,
				velocity: velocity.as_ref()?,
				phase: phase.as_ref()?,
			});
		}

		NvFlexUnmap(self.buffer);
		NvFlexUnmap(self.velocities);
		NvFlexUnmap(self.phases);

		Some(pvec)
	}

	pub fn flush(&mut self, solver: *mut NvFlexSolver) -> bool {
		if !self.has_changes {
			return false;
		}

		unsafe {
			NvFlexSetParticles(solver, self.buffer, std::ptr::null_mut());
			NvFlexSetVelocities(solver, self.velocities, std::ptr::null_mut());
			NvFlexSetPhases(solver, self.phases, std::ptr::null_mut());
			NvFlexSetActive(solver, self.active_indices, std::ptr::null_mut());
			NvFlexSetActiveCount(solver, self.get_active_count() as i32); // All are active for now.
		}

		self.has_changes = false;

		true
	}

	/// Creates an environment to safely and efficiently create new particles.
	/// They will be properly mapped and unmapped, however, you still need to [flush] these changes.
	pub fn factory<F: Fn(&mut factory::ParticleFactory)>(&mut self, generator: F) {
		let mut factory = unsafe {
			let particles = NvFlexMap(self.buffer, eNvFlexMapWait) as *mut Vector4;
			let velocities = NvFlexMap(self.velocities, eNvFlexMapWait) as *mut Vector3;
			let phases = NvFlexMap(self.phases, eNvFlexMapWait) as *mut i32;
			let active_indices = NvFlexMap(self.active_indices, eNvFlexMapWait) as *mut i32;

			factory::ParticleFactory::new(None, particles, velocities, phases, active_indices)
		};

		generator(&mut factory);

		if factory.nparticles > 0 {
			let count = self.get_count();
			for i in 0 .. factory.nparticles {
				self.particles.push( (count + i, true) );
			}
			self.has_changes = true;
		}

		self.unmap();
	}
}

impl Drop for ParticleState {
	fn drop(&mut self) {
		unsafe {
			NvFlexFreeBuffer(self.buffer);
			NvFlexFreeBuffer(self.velocities);
			NvFlexFreeBuffer(self.phases);
			NvFlexFreeBuffer(self.active_indices);
		}
	}
}

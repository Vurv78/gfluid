use std::mem::MaybeUninit;
use std::time::Instant;

// State holding all of the data for FleX.
use crate::{
	config,
	helper::*,
	types::{Particle, Quat, Vector3, Vector4},
};
use nvflex_sys::*;

mod geometry;
use geometry::{ShapeState, TriangleState};

mod particle;
use particle::ParticleState;

#[derive(Debug)]
pub struct FlexState {
	/* Shared */
	instant: Instant,
	lib: *mut NvFlexLibrary,

	/// Note this will most likely be null.
	desc: *mut NvFlexInitDesc,

	solver_desc: NvFlexSolverDesc,
	pub solver: *mut NvFlexSolver,

	pub particles: ParticleState,

	pub shapes: ShapeState,
	pub triangles: TriangleState,
}

impl FlexState {
	pub unsafe fn new(error_handler: Option<unsafe extern "C" fn(type_: NvFlexErrorSeverity, msg: *const i8, file: *const i8, line: i32)>) -> Self {
		use std::mem::size_of;

		let flex = NvFlexInit(NV_FLEX_VERSION as i32, error_handler, std::ptr::null_mut());
		if flex.is_null() {
			// Should never happen. If this does happen this should return a Result<Self, E> again
			panic!("Failed to create Flex Library");
		}

		// Create default solver settings
		let mut solver_desc = MaybeUninit::<NvFlexSolverDesc>::uninit();
		NvFlexSetSolverDescDefaults(solver_desc.as_mut_ptr());
		let solver_desc = solver_desc.assume_init();

		let solver = NvFlexCreateSolver(flex, &solver_desc);
		let particles = ParticleState::new(flex, config::MAX_PARTICLES);
		let shapes = ShapeState::new(flex, config::MAX_SHAPES);
		let triangles = TriangleState::new(flex, config::MAX_TRIANGLES);

		NvFlexSetParams(solver, &config::PARAMS);

		Self {
			instant: Instant::now(),
			lib: flex,

			desc: std::ptr::null_mut(),

			solver_desc,
			solver,

			particles,

			shapes,
			triangles,
		}
	}

	/// Loads default objects / scene
	pub unsafe fn init(&mut self) {
		let baux = NvFlexCollisionGeometry {
			box_: NvFlexBoxGeometry {
				halfExtents: [50000.0, 50000.0, 5.0],
			},
		};

		let flag = NvFlexMakeShapeFlags(eNvFlexShapeBox, false);
		self.shapes.create(
			baux,
			Vector4(0.0, 0.0, 0.0, 0.0),
			Quat(0.0, 0.0, 0.0, 0.0),
			flag,
		);

		self.shapes.create(
			baux,
			Vector4(0.0, 0.0, 0.0, 0.0),
			Quat(0.0, 1.0, 0.0, 0.0),
			flag,
		);

		self.shapes.create(
			baux,
			Vector4(0.0, 0.0, 0.0, 0.0),
			Quat(1.0, 0.0, 0.0, 0.0),
			flag,
		);

		let fluid = NvFlexMakePhase(0, eNvFlexPhaseSelfCollide | eNvFlexPhaseFluid);

		self.particles.factory(|mut x| {
			for i in 0..config::MAX_PARTICLES {
				x.create(
					Vector4(50.0 * i as f32, 0.0, 5000.0, 2.0),
					Vector3(0.0, 0.0, -5.0),
					fluid,
					true,
				);
			}
		});

		// This will call all of the NvFlexSet* functions
		self.particles.flush(self.solver);
		self.shapes.flush(self.solver);
		self.triangles.flush(self.solver);
	}

	pub fn tick(&mut self) {
		unsafe {
			let dt = self.instant.elapsed();
			self.instant = Instant::now();

			NvFlexUpdateSolver(self.solver, dt.as_secs_f32(), 1, false);
		}
	}

	pub unsafe fn get(&self) -> Option<Vec<Particle>> {
		self.particles.get(self.solver)
	}
}

impl Drop for FlexState {
	/// Consumes the FlexState, properly releasing allocated resources.
	fn drop(&mut self) {
		unsafe {
			NvFlexDestroySolver(self.solver);
			NvFlexShutdown(self.lib);
		}
	}
}

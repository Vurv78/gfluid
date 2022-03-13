use nvflex_sys::*;

use crate::types::{Vector4, Quat};

pub const MAX_PARTICLES: usize = 2000;
pub const MAX_SHAPES: usize = 1000;
pub const MAX_TRIANGLES: i32 = 1000;

/// 32  =  2' 0"    ≈     60cm     width & length
/// 36  =  2' 3"    ≈     70cm     height crouching
/// 72  =  4' 6"    ≈    135cm     height standing
pub const CAPSULE_RADIUS: f32 = 32.0;
pub const CAPSULE_HEIGHT: f32 = 72.0;

pub const PARTICLE_RADIUS: f32 = 20.0;

pub const BASEPLATE_SIZE: [f32; 3] = [5000.0, 5000.0, 5.0];
pub const BASEPLATE: Vector4 = Vector4(0.0, 0.0, -11136.0, 1.0);
pub const BASEPLATE_ROT: Quat = Quat(1.0, 0.0, 0.0, 0.0);

pub const PARAMS: NvFlexParams = NvFlexParams {
	numIterations: 2,
	gravity: [0.0, 0.0, -9.8],
	radius: PARTICLE_RADIUS, // The maximum interaction radius for particles
	solidRestDistance: 0.0,
	fluidRestDistance: 0.0,
	dynamicFriction: 0.0,
	staticFriction: 0.0,
	particleFriction: 0.0,
	restitution: 0.0,
	adhesion: 0.0,
	sleepThreshold: 0.0,
	maxSpeed: f32::MAX,
	maxAcceleration: 100.0, // 10x gravity
	shockPropagation: 0.0,
	dissipation: 0.0,
	damping: 0.0,
	wind: [0.0, 0.0, 0.0],
	drag: 0.0,
	lift: 0.0,
	cohesion: 0.025,
	surfaceTension: 0.0,
	viscosity: 0.0,
	vorticityConfinement: 40.0,
	anisotropyScale: 20.0,
	anisotropyMin: 0.1,
	anisotropyMax: 2.0,
	smoothing: 1.0,
	solidPressure: 1.0,
	freeSurfaceDrag: 0.0,
	buoyancy: 1.0,
	diffuseThreshold: f32::MAX,
	diffuseBuoyancy: 1.0,
	diffuseDrag: 0.8,
	diffuseBallistic: 16,
	diffuseLifetime: 2.0,
	collisionDistance: 0.025, // Distance particles maintain against shapes, note that for robust collision against triangle meshes this distance should be greater than zero
	particleCollisionMargin: 0.01,
	shapeCollisionMargin: 0.01,
	planes: [
		[0.0, 0.0, 0.0, 0.0],
		[0.0, 0.0, 0.0, 0.0],
		[0.0, 0.0, 0.0, 0.0],
		[0.0, 0.0, 0.0, 0.0],
		[0.0, 0.0, 0.0, 0.0],
		[0.0, 0.0, 0.0, 0.0],
		[0.0, 0.0, 0.0, 0.0],
		[0.0, 0.0, 0.0, 0.0],
	],
	numPlanes: 0,
	relaxationMode: nvflex_sys::eNvFlexRelaxationLocal,
	relaxationFactor: 1.0,
};

const MAX_PLANES: i32 = 12;

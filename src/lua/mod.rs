// Lua interface for gfluid
use rglua::{prelude::*, lua};
use crate::STATE;

use crate::state::{FlexState, Cube};
use crate::{
	config,
	helper::*,
	types::{Particle, Quat, Vector3, Vector4},
};

use nvflex_sys::*;
use std::sync::atomic::Ordering;

#[derive(Debug, thiserror::Error)]
pub enum GenericError {
	#[error("Couldn't get global FleX state.")]
	NoState
}

pub fn get_global_state<'flex>() -> Result<&'flex mut FlexState, GenericError> {
	let ptr = STATE.load(Ordering::Relaxed);
	unsafe { ptr.as_mut() }.ok_or(GenericError::NoState)
}

#[lua_function]
pub fn get_particles(l: LuaState) -> Result<i32, GenericError> {
	let state = get_global_state()?;

	if let Some(data) = unsafe { state.particles.get(state.solver) } {
		lua_createtable(l, data.len() as i32, 0);
		for (i, particle) in data.iter().enumerate() {
			lua_createtable(l, 0, 4); // -3 particle = {}

			lua_pushstring(l, cstr!("phase")); // -2
			lua_pushnumber(l, *particle.phase as f64); // -1
			lua_rawset(l, -3);

			lua_pushstring(l, cstr!("imass")); // -2
			lua_pushnumber(l, particle.pdata.3 as f64); // -1
			lua_rawset(l, -3);

			lua_pushstring(l, cstr!("velocity"));
			lua_pushvector(l, Vector::new( particle.velocity.0, particle.velocity.1, particle.velocity.2 )); // -2
			lua_rawset(l, -3); // t.velocity = stack[ #stack - 1 ]

			lua_pushstring(l, cstr!("position"));
			lua_pushvector(l, Vector::new( particle.pdata.0, particle.pdata.1, particle.pdata.2 )); // -2
			lua_rawset(l, -3); // t.position = stack[ #stack - 1 ]

			lua_rawseti(l, -2, i as i32 + 1); // particles[i + 1] = stack[#stack] (aka particle)
		}
		return Ok(1);
	}

	state.particles.unmap();
	Ok(0)
}

#[derive(Debug, thiserror::Error)]
enum CreateShapeError {
	#[error("Invalid shape kind: `{0}`")]
	InvalidShapeKind(isize),

	#[error("Failed to create: {0}")]
	Create(#[from] crate::state::CreateError),

	#[error("{0}")]
	Generic(#[from] GenericError)
}

#[lua_function]
fn create_box(l: LuaState) -> Result<i32, CreateShapeError> {
	let pos = luaL_checkvector(l, 1);
	let obbs = luaL_checkvector(l, 2);
	luaL_checktype(l, 3, TTABLE);

	lua_rawgeti(l, 3, 1);
	let x = luaL_optnumber(l, -1, 0.0) as f32;

	lua_rawgeti(l, 3, 2);
	let y = luaL_optnumber(l, -1, 0.0) as f32;

	lua_rawgeti(l, 3, 3);
	let z = luaL_optnumber(l, -1, 0.0) as f32;

	lua_rawgeti(l, 3, 4);
	let w = luaL_optnumber(l, -1, 0.0) as f32;


	let state = get_global_state()?;

	let the_box = Cube::new(Vector4(pos.x, pos.y, pos.z, 0.0), Quat(x, y, z, w), [obbs.x, obbs.y, obbs.z] );
	state.shapes.register(the_box.into())?;

	Ok(0)
}

#[lua_function]
fn create_particle(l: LuaState) -> Result<i32, GenericError> {
	let state = get_global_state()?;

	let pos = luaL_checkvector(l, 1);
	let velocity = luaL_checkvector(l, 2);
	let imass = luaL_optnumber(l, 3, 2.0);

	let fluid = NvFlexMakePhase(0, eNvFlexPhaseSelfCollide | eNvFlexPhaseFluid);

	state.particles.create( Vector4(pos.x, pos.y, pos.z, imass as f32), Vector3(velocity.x, velocity.y, velocity.z), fluid, true );
	state.particles.flush(state.solver);

	Ok(1)
}

#[lua_function]
fn flush(l: LuaState) -> Result<i32, GenericError> {
	let state = get_global_state()?;

	state.particles.flush(state.solver);

	Ok(0)
}

#[lua_function]
fn tick(_l: LuaState) -> i32 {
	if let Ok(flex) = get_global_state() {
		flex.tick();
	}

	0
}

#[lua_function]
fn get_boxes(l: LuaState) -> Result<i32, GenericError> {
	let state = get_global_state()?;

	let shapes = state.shapes.get_list();
	lua_createtable(l, shapes.len() as i32, 0);

	for (k, shape) in shapes.iter().enumerate() {
		lua_pushinteger(l, shape.kind() as isize);
		lua_setfield(l, -2, cstr!("kind"));

		lua_pushvector(l, shape.get_pos().to_owned().into());
		lua_setfield(l, -2, cstr!("pos"));

		lua_createtable(l, 4, 0);
		let quat = shape.get_rot();

		lua_pushnumber(l, quat.0 as f64);
		lua_rawseti(l, -2, 1);

		lua_pushnumber(l, quat.1 as f64);
		lua_rawseti(l, -2, 2);

		lua_pushnumber(l, quat.2 as f64);
		lua_rawseti(l, -2, 3);

		lua_pushnumber(l, quat.3 as f64);
		lua_rawseti(l, -2, 4);

		lua_setfield(l, -2, cstr!("rot"));
	}

	Ok(1)
}

/*#[lua_function]
fn particle_factory(l: LuaState) -> i32 {
	let state = STATE.load(Ordering::Relaxed);
	if let Some(state) = unsafe { state.as_mut() } {
		luaL_checktype(l, 1, TFUNCTION);

		state.particles.factory(|factory| {
			lua_pushcfunction(l, create_particle_fast);
			lua_call(l, 1, 0);
		});

		state.particles.flush(state.solver);

		return 1;
	}
	0
}*/

pub fn load(l: LuaState) {
	let r = reg! [
		// function getParticles() -> array<Particle>
		"getParticles" => get_particles,
		"getBoxes" => get_boxes,
		// function createShape() -> boolean

		"createBox" => create_box,
		// "createShape" => create_shape,
		"createParticle" => create_particle,

		"flush" => flush

		//"particleFactory" => particle_factory
	];

	lua_getglobal(l, cstr!("hook"));
	lua_getfield(l, -1, cstr!("Add"));

	lua_remove(l, -2); // Remove 'hook' table from stack

	// Push arguments to hook.Add
	lua_pushstring(l, cstr!("Tick"));
	lua_pushstring(l, cstr!("GFluid_Tick"));
	lua_pushcfunction(l, tick);

	// Call hook.Add
	lua_call(l, 3, 0);

	luaL_register(l, cstr!("flex"), r.as_ptr());
}
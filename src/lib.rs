#![allow(unused)]

use rglua::prelude::*;
use std::sync::atomic::{AtomicPtr, Ordering};

mod lua;
mod config;
mod helper;
mod state;
mod types;

use state::FlexState;

static STATE: AtomicPtr<FlexState> = AtomicPtr::new(std::ptr::null_mut());

#[gmod_open]
fn main(l: LuaState) -> i32 {
	unsafe extern "C" fn error_handler(severity: i32, msg: LuaString, file: LuaString, line: i32) {
		// In the future maybe this can cause a lua error?
		eprintln!("Flex Error!: Severity: {}, Msg: [{}], File: [{}], line: [{}]", severity, rstr!(msg), rstr!(file), line);
	}

	let flex_state = unsafe {
		let mut flex = Box::new(FlexState::new(Some(error_handler)));
		flex.init();
		flex
	};

	let flex_ptr = Box::into_raw(flex_state);
	STATE.store(flex_ptr, Ordering::Relaxed);

	lua::load(l);

	0
}

#[gmod_close]
fn close(_l: LuaState) -> i32 {
	let ptr = STATE.load(Ordering::SeqCst);
	// Get box out of pointer to interact with flex state
	// This will be dropped automagically
	let flex_state = unsafe { Box::from_raw(ptr as *mut FlexState) };

	// All cleanup work will be done here
	std::mem::drop(flex_state);
	0
}

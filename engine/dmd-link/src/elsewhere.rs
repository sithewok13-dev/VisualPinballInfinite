// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2026, the Visual Pinball Infinite contributors

//! Elsewhere, stand-ins for the Windows calls `win.rs` declares: no link is made (`Link::create`
//! and `Link::open` say so before any is called), and the clock is the system's own.

#![allow(non_snake_case, dead_code, clippy::missing_safety_doc)]

use std::ffi::c_void;

pub type Handle = *mut c_void;

pub const INVALID_HANDLE_VALUE: Handle = -1isize as Handle;
pub const PAGE_READWRITE: u32 = 0x04;
pub const FILE_MAP_ALL_ACCESS: u32 = 0x000F_001F;
pub const EVENT_MODIFY_STATE: u32 = 0x0002;
pub const SYNCHRONIZE: u32 = 0x0010_0000;
pub const ERROR_ALREADY_EXISTS: i32 = 183;

pub unsafe fn CreateFileMappingW(_file: Handle, _attributes: *const c_void, _protect: u32, _size_high: u32, _size_low: u32, _name: *const u16) -> Handle {
    std::ptr::null_mut()
}

pub unsafe fn OpenFileMappingW(_access: u32, _inherit: i32, _name: *const u16) -> Handle {
    std::ptr::null_mut()
}

pub unsafe fn MapViewOfFile(_mapping: Handle, _access: u32, _offset_high: u32, _offset_low: u32, _bytes: usize) -> *mut c_void {
    std::ptr::null_mut()
}

pub unsafe fn UnmapViewOfFile(_address: *const c_void) -> i32 {
    0
}

pub unsafe fn CreateEventW(_attributes: *const c_void, _manual_reset: i32, _initial_state: i32, _name: *const u16) -> Handle {
    std::ptr::null_mut()
}

pub unsafe fn OpenEventW(_access: u32, _inherit: i32, _name: *const u16) -> Handle {
    std::ptr::null_mut()
}

pub unsafe fn SetEvent(_event: Handle) -> i32 {
    0
}

pub unsafe fn CloseHandle(_handle: Handle) -> i32 {
    0
}

/// `s` as a null-terminated UTF-16 string.
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

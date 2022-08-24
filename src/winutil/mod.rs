use std::ffi::CString;
use std::mem::size_of;
use std::ptr;

use windows::Win32::Security::SECURITY_ATTRIBUTES;

use windows::Win32::System::Threading::{
    CreateProcessA, CREATE_NEW_CONSOLE, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTUPINFOA,
};

use windows::core::{PCSTR, PSTR};

pub mod structs;
use structs::*;

pub fn create_process_param(path: String, args: String, params: CreationParams) {
    let path_cstr = PCSTR(CString::new(path).unwrap().into_raw() as *const u8);
    let args_cstr = PSTR(CString::new(args).unwrap().into_raw() as *mut u8);

    let native_params = params.as_native();

    unsafe {
        CreateProcessA(
            path_cstr,
            args_cstr,
            &native_params.process_attributes,
            &native_params.thread_attributes,
            native_params.inherit_handles,
            native_params.creation_flags,
            native_params.environment,
            native_params.current_directory,
            &native_params.startup_info,
            &mut PROCESS_INFORMATION::default(),
        );
    }
}

pub fn create_process(path: String, args: String) {
    let path_cstr = PCSTR(CString::new(path).unwrap().into_raw() as *const u8);
    let args_cstr = PSTR(CString::new(args).unwrap().into_raw() as *mut u8);

    // let working_directory = match override_wd {
    //     Some(wd) => PCSTR(CString::new(wd).unwrap().into_raw() as *const u8),
    //     None => PCSTR::null(),
    // };

    unsafe {
        CreateProcessA(
            path_cstr,
            args_cstr,
            &SECURITY_ATTRIBUTES::default(),
            &SECURITY_ATTRIBUTES::default(),
            false,
            PROCESS_CREATION_FLAGS::default() | CREATE_NEW_CONSOLE,
            ptr::null(),
            PCSTR::null(), // working_directory,
            &STARTUPINFOA {
                cb: size_of::<STARTUPINFOA>() as u32,
                ..Default::default()
            },
            &mut PROCESS_INFORMATION::default(),
        );
    }
}

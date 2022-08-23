use std::ffi::CString;
use std::mem::size_of;
use std::ptr;

use windows::core::{ PCSTR, PSTR };

use windows::Win32::System::Threading::{
    CreateProcessA,
    PROCESS_INFORMATION,
    STARTUPINFOA,
    PROCESS_CREATION_FLAGS,
    CREATE_NEW_CONSOLE
};

use windows::Win32::Security::SECURITY_ATTRIBUTES;

pub fn create_process(path: String, args: String, override_wd: Option<String>) {
    let path_cstr = PCSTR(CString::new(path).unwrap().into_raw() as *const u8);
    let args_cstr = PSTR(CString::new(args).unwrap().into_raw()  as *mut u8);

    let working_directory = match override_wd {
        Some(wd) => PCSTR(CString::new(wd).unwrap().into_raw() as *const u8),
        None => PCSTR::null()
    };

    unsafe {
        CreateProcessA(
            path_cstr,
            args_cstr,
            &SECURITY_ATTRIBUTES::default(),
            &SECURITY_ATTRIBUTES::default(),
            false,
            PROCESS_CREATION_FLAGS::default() | CREATE_NEW_CONSOLE,
            ptr::null(),
            working_directory,
            &STARTUPINFOA { cb: size_of::<STARTUPINFOA>() as u32, ..Default::default() },
            &mut PROCESS_INFORMATION::default()
        );
    }
}

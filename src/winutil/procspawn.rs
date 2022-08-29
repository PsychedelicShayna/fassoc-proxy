use std::ffi::CString;
use windows::core::{PCSTR, PSTR};
use windows::Win32::System::Threading::{CreateProcessA, PROCESS_INFORMATION};

use super::structs::*;

pub fn create_process(path: String, args: String, cd: String, extras: NativeCreationExtras) {
    let path_cstr = PCSTR(match CString::new(path.to_owned()) {
        Ok(cstr) => cstr.into_raw() as *mut u8,
        Err(error) => {
            log::error!(
                "Could not turn the command string ({}) into a native CString, encountered error: {}",
                path,
                error
            );
            panic!();
        }
    });

    let args_cstr = PSTR(match CString::new(args.to_owned()) {
        Ok(cstr) => cstr.into_raw() as *mut u8,
        Err(error) => {
            log::error!(
                "Could not turn the argument string ({}) into a native CString, encountered error: {}",
                args,
                error
            );
            panic!();
        }
    });

    let cd_cstr = PCSTR(match CString::new(cd.to_owned()) {
        Ok(cstr) => cstr.into_raw() as *mut u8,
        Err(error) => {
            log::error!(
                "Could not turn the current directory / cd string ({}) string into a native CString, encountered error: {}",
                args,
                error
            );
            panic!();
        }
    });

    unsafe {
        let mut process_information = PROCESS_INFORMATION::default();

        let result = CreateProcessA(
            path_cstr,
            args_cstr,
            &extras.process_attributes,
            &extras.thread_attributes,
            extras.inherit_handles,
            extras.creation_flags,
            extras.environment,
            cd_cstr,
            &extras.startup_info,
            &mut process_information,
        );

        if result.0 == 0 {
            log::error!("WinAPI reported that the process creation failed (result == 0)");
        }

        log::debug!("CreateProcessA returned: {:?}", result);
        log::debug!("Process Information ---------\n{:?}", process_information);
    }
}

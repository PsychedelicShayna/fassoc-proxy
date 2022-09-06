use std::ffi::{c_void, CString};
use windows::core::{PCSTR, PSTR};
use windows::Win32::System::Threading::{CreateProcessA, PROCESS_INFORMATION};

use windows::Win32::UI::WindowsAndMessaging::{
    SHOW_WINDOW_CMD, SW_HIDE, SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, SW_SHOW, SW_SHOWDEFAULT,
    SW_SHOWMAXIMIZED, SW_SHOWMINIMIZED, SW_SHOWMINNOACTIVE, SW_SHOWNA, SW_SHOWNOACTIVATE,
    SW_SHOWNORMAL,
};

use super::rules::Command;
use std::collections::HashMap;
use std::mem::size_of;
use std::ptr;
use windows::Win32::Foundation::BOOL;
use windows::Win32::Security::SECURITY_ATTRIBUTES;

use windows::Win32::System::Console::{
    BACKGROUND_BLUE, BACKGROUND_GREEN, BACKGROUND_INTENSITY, BACKGROUND_RED,
    COMMON_LVB_GRID_HORIZONTAL, COMMON_LVB_GRID_LVERTICAL, COMMON_LVB_GRID_RVERTICAL,
    COMMON_LVB_LEADING_BYTE, COMMON_LVB_REVERSE_VIDEO, COMMON_LVB_SBCSDBCS,
    COMMON_LVB_TRAILING_BYTE, COMMON_LVB_UNDERSCORE, FOREGROUND_BLUE, FOREGROUND_GREEN,
    FOREGROUND_INTENSITY, FOREGROUND_RED,
};

use windows::Win32::System::Threading::{
    CREATE_BREAKAWAY_FROM_JOB, CREATE_DEFAULT_ERROR_MODE, CREATE_NEW_CONSOLE,
    CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, CREATE_PRESERVE_CODE_AUTHZ_LEVEL,
    CREATE_PROTECTED_PROCESS, CREATE_SECURE_PROCESS, CREATE_SEPARATE_WOW_VDM,
    CREATE_SHARED_WOW_VDM, CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT, DEBUG_ONLY_THIS_PROCESS,
    DEBUG_PROCESS, DETACHED_PROCESS, EXTENDED_STARTUPINFO_PRESENT, INHERIT_PARENT_AFFINITY,
    PROCESS_CREATION_FLAGS, STARTF_FORCEOFFFEEDBACK, STARTF_FORCEONFEEDBACK, STARTF_PREVENTPINNING,
    STARTF_RUNFULLSCREEN, STARTF_TITLEISAPPID, STARTF_TITLEISLINKNAME, STARTF_UNTRUSTEDSOURCE,
    STARTF_USECOUNTCHARS, STARTF_USEFILLATTRIBUTE, STARTF_USEHOTKEY, STARTF_USEPOSITION,
    STARTF_USESHOWWINDOW, STARTF_USESIZE, STARTF_USESTDHANDLES, STARTUPINFOA, STARTUPINFOW_FLAGS,
};

#[derive(Debug)]
pub struct ProcessCreationParameters {
    pub command: PCSTR,
    pub arguments: PSTR,
    pub cwd: PCSTR,
    pub process_attributes: SECURITY_ATTRIBUTES,
    pub thread_attributes: SECURITY_ATTRIBUTES,
    pub inherit_handles: BOOL,
    pub creation_flags: PROCESS_CREATION_FLAGS,
    pub environment: *const c_void,
    pub current_directory: PCSTR,
    pub startup_info: STARTUPINFOA,
}

impl Default for ProcessCreationParameters {
    fn default() -> ProcessCreationParameters {
        ProcessCreationParameters {
            command: PCSTR::null(),
            arguments: PSTR::null(),
            cwd: PCSTR::null(),
            process_attributes: SECURITY_ATTRIBUTES::default(),
            thread_attributes: SECURITY_ATTRIBUTES::default(),
            inherit_handles: BOOL(0),
            creation_flags: PROCESS_CREATION_FLAGS::default(),
            environment: ptr::null(),
            current_directory: PCSTR::null(),
            startup_info: STARTUPINFOA {
                cb: size_of::<STARTUPINFOA>() as u32,
                ..Default::default()
            },
        }
    }
}

impl ProcessCreationParameters {
    pub fn from_rule(rule: &Command) -> ProcessCreationParameters {
        let mut pcp = ProcessCreationParameters::default();

        pcp.command = CString::new(rule.path.to_owned()).map_or_else(
            |error| {
                log::error!("Couldn't nativize the command string \"{}\" from the selected rule, due to error: {}", 
                    rule.path, error);

                panic!();
            },
            |cstr| PCSTR(cstr.into_raw() as *mut u8),
        );

        rule.arguments.to_owned().map(|args| {
            pcp.arguments = CString::new(args.to_owned()).map_or_else(
                |error| {
                    log::error!("Couldn't nativize the arguments string \"{}\" from the selected rule, due to error: {}", 
                        args, error);

                    panic!();
                },
                |cstr| PSTR(cstr.into_raw() as *mut u8),
            );
        });

        rule.cwd.to_owned().map(|cwd| {
            pcp.cwd = CString::new(cwd.to_owned()).map_or_else(
                |error| {
                    log::error!("Couldn't nativize the cwd string \"{}\" from the selected rule, due to error: {}", 
                        cwd, error);

                    panic!();
                },
                |cstr| PCSTR(cstr.into_raw() as *mut u8),
            );
        });

        pcp.thread_attributes = SECURITY_ATTRIBUTES::default();

        rule.thread_attributes.to_owned().map(|attr| {
            let mut native = SECURITY_ATTRIBUTES {
                nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
                ..Default::default()
            };

            attr.inherit_handle.map(|inherit_handle| {
                native.bInheritHandle = BOOL(if inherit_handle { 1 } else { 0 });
            });

            attr.security_descriptor.map(|security_descriptor| {
                native.lpSecurityDescriptor = security_descriptor as *mut c_void;
            });

            pcp.thread_attributes = native;
        });

        rule.process_attributes.to_owned().map(|attr| {
            let mut native = SECURITY_ATTRIBUTES {
                nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
                ..Default::default()
            };

            attr.inherit_handle.map(|inherit_handle| {
                native.bInheritHandle = BOOL(if inherit_handle { 1 } else { 0 });
            });

            attr.security_descriptor.map(|security_descriptor| {
                native.lpSecurityDescriptor = security_descriptor as *mut c_void;
            });

            pcp.process_attributes = native;
        });

        rule.inherit_handles.to_owned().map(|inherit_handles| {
            pcp.inherit_handles = BOOL(if inherit_handles { 1 } else { 0 });
        });

        rule.creation_flags.to_owned().map(|creation_flags| {
            let resolver = std::collections::HashMap::<&str, PROCESS_CREATION_FLAGS>::from([
                ("CREATE_BREAKAWAY_FROM_JOB", CREATE_BREAKAWAY_FROM_JOB),
                ("CREATE_DEFAULT_ERROR_MODE", CREATE_DEFAULT_ERROR_MODE),
                ("CREATE_NEW_CONSOLE", CREATE_NEW_CONSOLE),
                ("CREATE_NEW_PROCESS_GROUP", CREATE_NEW_PROCESS_GROUP),
                ("CREATE_NO_WINDOW", CREATE_NO_WINDOW),
                ("CREATE_PROTECTED_PROCESS", CREATE_PROTECTED_PROCESS),
                (
                    "CREATE_PRESERVE_CODE_AUTHZ_LEVEL",
                    CREATE_PRESERVE_CODE_AUTHZ_LEVEL,
                ),
                ("CREATE_SECURE_PROCESS", CREATE_SECURE_PROCESS),
                ("CREATE_SEPARATE_WOW_VDM", CREATE_SEPARATE_WOW_VDM),
                ("CREATE_SHARED_WOW_VDM", CREATE_SHARED_WOW_VDM),
                ("CREATE_SUSPENDED", CREATE_SUSPENDED),
                ("CREATE_UNICODE_ENVIRONMENT", CREATE_UNICODE_ENVIRONMENT),
                ("DEBUG_ONLY_THIS_PROCESS", DEBUG_ONLY_THIS_PROCESS),
                ("DEBUG_PROCESS", DEBUG_PROCESS),
                ("DETACHED_PROCESS", DETACHED_PROCESS),
                ("EXTENDED_STARTUPINFO_PRESENT", EXTENDED_STARTUPINFO_PRESENT),
                ("INHERIT_PARENT_AFFINITY", INHERIT_PARENT_AFFINITY),
            ]);

            let nativized_array = creation_flags.iter().filter_map(|e| {
                e.as_str().map_or(e.as_u64().map(|i| i as u32), |e| {
                    resolver.get(e).map(|flag| flag.0)
                })
            });

            let flags: u32 = nativized_array.fold(0, |acc, e| acc | e);

            pcp.creation_flags = PROCESS_CREATION_FLAGS(flags);
        });

        rule.extras.to_owned().map(|extras| {
            let mut native = STARTUPINFOA {
                cb: size_of::<STARTUPINFOA>() as u32,
                ..Default::default()
            };

            extras.desktop.to_owned().map(|desktop| {
                CString::new(desktop.to_owned())
                    .map(|cstr| native.lpDesktop = PSTR(cstr.into_raw() as *mut u8))
            });

            extras.title.to_owned().map(|title| {
                CString::new(title.to_owned())
                    .map(|cstr| native.lpTitle = PSTR(cstr.into_raw() as *mut u8))
            });

            extras.x.map(|x| native.dwX = x);
            extras.y.map(|y| native.dwY = y);
            extras.x_size.map(|x_size| native.dwXSize = x_size);
            extras.y_size.map(|y_size| native.dwYSize = y_size);
            extras
                .x_count_chars
                .map(|x_count_chars| native.dwXCountChars = x_count_chars);
            extras
                .y_count_chars
                .map(|y_count_chars| native.dwYCountChars = y_count_chars);

            extras.fill_attribute.map(|fill_attribute| {
                let resolver = HashMap::<&str, u32>::from([
                    ("FOREGROUND_BLUE", FOREGROUND_BLUE),
                    ("FOREGROUND_RED", FOREGROUND_RED),
                    ("FOREGROUND_GREEN", FOREGROUND_GREEN),
                    ("BACKGROUND_BLUE", BACKGROUND_BLUE),
                    ("BACKGROUND_RED", BACKGROUND_RED),
                    ("BACKGROUND_GREEN", BACKGROUND_GREEN),
                    ("BACKGROUND_INTENSITY", BACKGROUND_INTENSITY),
                    ("FOREGROUND_INTENSITY", FOREGROUND_INTENSITY),
                    ("COMMON_LVB_LEADING_BYTE", COMMON_LVB_LEADING_BYTE),
                    ("COMMON_LVB_TRAILING_BYT", COMMON_LVB_TRAILING_BYTE),
                    ("COMMON_LVB_GRID_HORIZONTAL", COMMON_LVB_GRID_HORIZONTAL),
                    ("COMMON_LVB_GRID_LVERTICAL", COMMON_LVB_GRID_LVERTICAL),
                    ("COMMON_LVB_GRID_RVERTICAL", COMMON_LVB_GRID_RVERTICAL),
                    ("COMMON_LVB_REVERSE_VIDEO", COMMON_LVB_REVERSE_VIDEO),
                    ("COMMON_LVB_UNDERSCORE", COMMON_LVB_UNDERSCORE),
                    ("COMMON_LVB_SBCSDBCS", COMMON_LVB_SBCSDBCS),
                ]);

                let nativized_array = fill_attribute.iter().filter_map(|e| {
                    e.as_str().map_or(e.as_u64().map(|i| i as u32), |e| {
                        resolver.get(e).map(|i| i.to_owned())
                    })
                });

                let flags: u32 = nativized_array.fold(0, |acc, e| acc | e);

                native.dwFillAttribute = flags;
            });

            extras.flags.map(|flags| {
                let resolver = std::collections::HashMap::<&str, STARTUPINFOW_FLAGS>::from([
                    ("STARTF_FORCEONFEEDBACK", STARTF_FORCEONFEEDBACK),
                    ("STARTF_FORCEOFFFEEDBACK", STARTF_FORCEOFFFEEDBACK),
                    ("STARTF_PREVENTPINNING", STARTF_PREVENTPINNING),
                    ("STARTF_RUNFULLSCREEN", STARTF_RUNFULLSCREEN),
                    ("STARTF_TITLEISAPPID", STARTF_TITLEISAPPID),
                    ("STARTF_TITLEISLINKNAME", STARTF_TITLEISLINKNAME),
                    ("STARTF_UNTRUSTEDSOURCE", STARTF_UNTRUSTEDSOURCE),
                    ("STARTF_USECOUNTCHARS", STARTF_USECOUNTCHARS),
                    ("STARTF_USEFILLATTRIBUTE", STARTF_USEFILLATTRIBUTE),
                    ("STARTF_USEHOTKEY", STARTF_USEHOTKEY),
                    ("STARTF_USEPOSITION", STARTF_USEPOSITION),
                    ("STARTF_USESHOWWINDOW", STARTF_USESHOWWINDOW),
                    ("STARTF_USESIZE", STARTF_USESIZE),
                    ("STARTF_USESTDHANDLES", STARTF_USESTDHANDLES),
                ]);

                let nativized_array = flags.iter().filter_map(|e| {
                    e.as_str().map_or(e.as_u64().map(|i| i as u32), |e| {
                        resolver.get(e).map(|i| i.0)
                    })
                });

                let flags: u32 = nativized_array.fold(0, |acc, e| acc | e);

                native.dwFlags = STARTUPINFOW_FLAGS(flags);
            });

            extras.show_window.map(|show_window| {
                let resolver = std::collections::HashMap::<&str, SHOW_WINDOW_CMD>::from([
                    ("SW_HIDE", SW_HIDE),
                    ("SW_MAXIMIZE", SW_MAXIMIZE),
                    ("SW_MINIMIZE", SW_MINIMIZE),
                    ("SW_RESTORE", SW_RESTORE),
                    ("SW_SHOW", SW_SHOW),
                    ("SW_SHOWDEFAULT", SW_SHOWDEFAULT),
                    ("SW_SHOWMAXIMIZED", SW_SHOWMAXIMIZED),
                    ("SW_SHOWMINIMIZED", SW_SHOWMINIMIZED),
                    ("SW_SHOWMINNOACTIVE", SW_SHOWMINNOACTIVE),
                    ("SW_SHOWNA", SW_SHOWNA),
                    ("SW_SHOWNOACTIVATE", SW_SHOWNOACTIVATE),
                    ("SW_SHOWNORMAL", SW_SHOWNORMAL),
                ]);

                let nativized_array = show_window.iter().filter_map(|e| {
                    e.as_str().map_or(e.as_u64().map(|i| i as u32), |e| {
                        resolver.get(e).map(|sw| sw.0)
                    })
                });

                let flags: u32 = nativized_array.fold(0, |acc, e| acc | e);

                native.wShowWindow = flags as u16;
            });
        });

        return pcp;
    }
}

#[derive(Debug)]
pub enum CreateProcessError {
    CommandNotExecutable(String),
    CommandDoesNotExist(String),
    CommandNotAbsolute(String),
}

impl std::fmt::Display for CreateProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateProcessError::CommandNotExecutable(cmd) => {
                write!(f, "The command \"{}\" is not executable", cmd)
            }
            CreateProcessError::CommandDoesNotExist(cmd) => {
                write!(f, "The command \"{}\" does not exist", cmd)
            }
            CreateProcessError::CommandNotAbsolute(cmd) => {
                write!(f, "The commaned \"{}\" does not have an absolute path", cmd)
            }
        }
    }
}

pub fn invoke_command(rule: &Command) -> Result<PROCESS_INFORMATION, CreateProcessError> {
    let params = ProcessCreationParameters::from_rule(rule);

    let command_path = std::path::Path::new(&rule.path);

    if !command_path.exists() {
        return Err(CreateProcessError::CommandDoesNotExist(rule.path.clone()));
    }

    if !command_path.extension().map_or(false, |ext| {
        ext.to_str().map_or(false, |exts| exts.ends_with("exe"))
    }) {
        return Err(CreateProcessError::CommandNotExecutable(
            rule.path.to_owned(),
        ));
    }

    if !command_path.is_absolute() {
        return Err(CreateProcessError::CommandNotAbsolute(rule.path.to_owned()));
    }

    unsafe {
        let mut process_information = PROCESS_INFORMATION::default();

        let result = CreateProcessA(
            params.command,
            params.arguments,
            &params.process_attributes,
            &params.thread_attributes,
            params.inherit_handles,
            params.creation_flags,
            params.environment,
            params.cwd,
            &params.startup_info,
            &mut process_information,
        );

        if result.0 == 0 {
            log::error!("WinAPI reported that the process creation failed (result == 0)");
        }

        log::debug!("CreateProcessA returned: {:?}", result);
        log::debug!("Process Information ---------\n{:?}", process_information);

        Ok(process_information)
    }
}

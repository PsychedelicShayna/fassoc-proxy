use serde_json as sj;
use std::collections::HashMap;
use std::ffi::{c_void, CString};
use std::mem::size_of;
use std::ptr;
use windows::core::{PCSTR, PSTR};
use windows::Win32::Foundation::{BOOL, HANDLE};
use windows::Win32::Security::SECURITY_ATTRIBUTES;
use windows::Win32::System::Console::*;
use windows::Win32::System::Threading::*;

// -----------------------------------------------------------------------------
// StartupInformation
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct StartupInformation {
    pub desktop: Option<String>,
    pub title: Option<String>,
    pub x: Option<u32>,
    pub y: Option<u32>,
    pub x_size: Option<u32>,
    pub y_size: Option<u32>,
    pub x_count_chars: Option<u32>,
    pub y_count_chars: Option<u32>,
    pub fill_attribute: Option<u32>,
    pub fill_attribute_append: Option<bool>,
    pub flags: Option<u32>,
    pub flags_append: Option<bool>,
    pub show_window: Option<u16>,
    pub stdin_handle: Option<isize>,
    pub stdout_handle: Option<isize>,
    pub stderr_handle: Option<isize>,
}

impl Default for StartupInformation {
    fn default() -> StartupInformation {
        StartupInformation {
            desktop: None,
            title: None,
            x: None,
            y: None,
            x_size: None,
            y_size: None,
            x_count_chars: None,
            y_count_chars: None,
            fill_attribute: None,
            fill_attribute_append: None,
            flags: None,
            flags_append: None,
            show_window: None,
            stdin_handle: None,
            stdout_handle: None,
            stderr_handle: None,
        }
    }
}

impl StartupInformation {
    pub fn fattr_flagstr_to_u32(flagstr: &str) -> Option<u32> {
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

        resolver.get(flagstr).map(|f| f.clone())
    }

    pub fn siwf_flagstr_to_u32(flagstr: &str) -> Option<u32> {
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

        resolver.get(flagstr).map(|f| f.0)
    }

    pub fn from_json(json: &sj::Map<String, sj::Value>) -> StartupInformation {
        StartupInformation {
            desktop: json
                .get("desktop")
                .and_then(|jv| jv.as_str())
                .map(|s| s.to_owned()),

            title: json
                .get("title")
                .and_then(|jv| jv.as_str())
                .map(|s| s.to_owned()),

            x: json.get("x").and_then(|jv| jv.as_u64()).map(|i| i as u32),
            y: json.get("y").and_then(|jv| jv.as_u64()).map(|i| i as u32),

            x_size: json
                .get("x_size")
                .and_then(|jv| jv.as_u64())
                .map(|i| i as u32),

            y_size: json
                .get("y_size")
                .and_then(|jv| jv.as_u64())
                .map(|i| i as u32),

            x_count_chars: json
                .get("x_count_chars")
                .and_then(|jv| jv.as_u64())
                .map(|i| i as u32),

            y_count_chars: json
                .get("y_count_chars")
                .and_then(|jv| jv.as_u64())
                .map(|i| i as u32),

            fill_attribute: json
                .get("fill_attribute")
                .and_then(|jv| jv.as_array())
                .map(|array| {
                    let nativized_array = array.iter().filter_map(|e| {
                        e.as_str().map_or(e.as_u64().map(|i| i as u32), |e| {
                            StartupInformation::fattr_flagstr_to_u32(e).map(|i| i as u32)
                        })
                    });

                    nativized_array.fold(0, |acc, e| acc | e)
                }),

            fill_attribute_append: json
                .get("fill_attribute_append")
                .and_then(|jv| jv.as_bool()),

            flags: json.get("flags").and_then(|jv| jv.as_array()).map(|array| {
                let nativized_array = array.iter().filter_map(|e| {
                    e.as_str().map_or(e.as_u64().map(|i| i as u32), |e| {
                        StartupInformation::siwf_flagstr_to_u32(e)
                    })
                });

                nativized_array.fold(0, |acc, e| acc | e)
            }),

            flags_append: json.get("flags_append").and_then(|jv| jv.as_bool()),

            show_window: json
                .get("show_window")
                .and_then(|jv| jv.as_u64())
                .map(|i| i as u16),

            stdin_handle: json
                .get("stdin_handle")
                .and_then(|jv| jv.as_u64())
                .map(|i| i as isize),
            stdout_handle: json
                .get("stdout_handle")
                .and_then(|jv| jv.as_u64())
                .map(|i| i as isize),
            stderr_handle: json
                .get("stderr_handle")
                .and_then(|jv| jv.as_u64())
                .map(|i| i as isize),

            ..Default::default()
        }
    }

    pub fn as_native(&self) -> STARTUPINFOA {
        let mut si = STARTUPINFOA {
            cb: size_of::<STARTUPINFOA>() as u32,
            ..Default::default()
        };

        self.desktop.as_ref().map(|str| {
            CString::new(str.to_owned()).map(|cstr| si.lpDesktop = PSTR(cstr.into_raw() as *mut u8))
        });

        self.title.as_ref().map(|str| {
            CString::new(str.to_owned()).map(|cstr| si.lpTitle = PSTR(cstr.into_raw() as *mut u8))
        });

        self.x.map(|x| si.dwX = x);
        self.y.map(|y| si.dwY = y);

        self.x_size.map(|x| si.dwXSize = x);
        self.y_size.map(|y| si.dwYSize = y);

        self.x_count_chars.map(|x| si.dwXCountChars = x);
        self.y_count_chars.map(|y| si.dwYCountChars = y);

        self.fill_attribute.map(|fattr| {
            if self.fill_attribute_append.unwrap_or(false) {
                si.dwFillAttribute |= fattr;
            } else {
                si.dwFillAttribute = fattr;
            }
        });

        self.flags.map(|flags| {
            if self.flags_append.unwrap_or(false) {
                si.dwFlags.0 |= flags;
            } else {
                si.dwFlags.0 = flags;
            }
        });

        self.show_window.map(|sw| si.wShowWindow = sw);

        self.stdin_handle.map(|h| si.hStdInput = HANDLE(h));
        self.stdout_handle.map(|h| si.hStdOutput = HANDLE(h));
        self.stderr_handle.map(|h| si.hStdError = HANDLE(h));

        return si;
    }
}

// -----------------------------------------------------------------------------
// SecurityAttributes
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct SecurityAttributes {
    pub security_descriptor: Option<isize>,
    pub inherit_handle: Option<bool>,
}

impl Default for SecurityAttributes {
    fn default() -> SecurityAttributes {
        SecurityAttributes {
            security_descriptor: None,
            inherit_handle: None,
        }
    }
}

impl SecurityAttributes {
    pub fn from_json(json: &sj::Map<String, sj::Value>) -> SecurityAttributes {
        SecurityAttributes {
            security_descriptor: json
                .get("security_descriptor")
                .and_then(|jv| jv.as_u64())
                .map(|i| i as isize),

            inherit_handle: json.get("inherit_handle").and_then(|jv| jv.as_bool()),
            ..Default::default()
        }
    }

    pub fn as_native(&self) -> SECURITY_ATTRIBUTES {
        let mut sa = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            ..Default::default()
        };

        self.security_descriptor
            .map(|sd| sa.lpSecurityDescriptor = sd as *mut c_void);

        self.inherit_handle
            .map(|ih| sa.bInheritHandle = BOOL(if ih { 1 } else { 0 }));

        return sa;
    }
}

// -----------------------------------------------------------------------------
// NativeCreationExtras
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct NativeCreationExtras {
    pub process_attributes: SECURITY_ATTRIBUTES,
    pub thread_attributes: SECURITY_ATTRIBUTES,
    pub inherit_handles: BOOL,
    pub creation_flags: PROCESS_CREATION_FLAGS,
    pub environment: *const c_void,
    pub current_directory: PCSTR,
    pub startup_info: STARTUPINFOA,
}

impl Default for NativeCreationExtras {
    fn default() -> NativeCreationExtras {
        NativeCreationExtras {
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

// -----------------------------------------------------------------------------
// CreationExtras
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct CreationExtras {
    pub process_attributes: Option<SecurityAttributes>,
    pub thread_attributes: Option<SecurityAttributes>,
    pub inherit_handles: Option<bool>,
    pub creation_flags: Option<u32>,
    pub creation_flags_append: Option<bool>,
    pub environment: Option<isize>,
    pub startup_info: Option<StartupInformation>,
}

impl Default for CreationExtras {
    fn default() -> CreationExtras {
        CreationExtras {
            process_attributes: None,
            thread_attributes: None,
            inherit_handles: None,
            creation_flags: None,
            creation_flags_append: None,
            environment: None,
            startup_info: None,
        }
    }
}

impl CreationExtras {
    pub fn cf_flagstr_to_u32(flagstr: &str) -> Option<u32> {
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

        resolver.get(flagstr).map(|s| s.0)
    }

    pub fn from_json(json: &sj::Map<String, sj::Value>) -> CreationExtras {
        let mut creation_params = CreationExtras::default();

        creation_params.process_attributes = json
            .get("process_attributes")
            .and_then(|jv| jv.as_object())
            .map(|obj| SecurityAttributes::from_json(obj));

        creation_params.thread_attributes = json
            .get("thread_attributes")
            .and_then(|jv| jv.as_object())
            .map(|obj| SecurityAttributes::from_json(obj));

        creation_params.inherit_handles = json.get("inherit_handles").and_then(|jv| jv.as_bool());

        creation_params.creation_flags = json
            .get("creation_flags")
            .and_then(|jv| jv.as_array())
            .map(|array| {
                let nativized_array = array.iter().filter_map(|e| {
                    e.as_str().map_or(e.as_u64().map(|i| i as u32), |e| {
                        CreationExtras::cf_flagstr_to_u32(e)
                    })
                });

                nativized_array.fold(0, |acc, e| acc | e)
            });

        creation_params.creation_flags_append = json
            .get("creation_flags_append")
            .and_then(|jv| jv.as_bool());

        creation_params.environment = json
            .get("environment")
            .and_then(|jv| jv.as_i64())
            .map(|i| i as isize);

        creation_params.startup_info = json
            .get("startup_info")
            .and_then(|jv| jv.as_object())
            .map(|o| StartupInformation::from_json(o));

        return creation_params;
    }

    pub fn as_native(&self) -> NativeCreationExtras {
        let mut np = NativeCreationExtras::default();

        self.process_attributes
            .as_ref()
            .map(|pa| np.process_attributes = pa.as_native());

        self.thread_attributes
            .as_ref()
            .map(|ta| np.thread_attributes = ta.as_native());

        self.inherit_handles
            .map(|ih| np.inherit_handles = BOOL(if ih { 1 } else { 0 }));

        self.creation_flags.map(|cf| {
            np.creation_flags = PROCESS_CREATION_FLAGS(cf);
        });

        self.environment.map(|e| {
            np.environment = e as *mut c_void;
        });

        self.startup_info.as_ref().map(|si| {
            np.startup_info = si.as_native();
        });

        return np;
    }
}

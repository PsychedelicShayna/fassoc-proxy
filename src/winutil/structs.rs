use std::ffi::{c_void, CString};
use std::mem::size_of;
use std::ptr;
use windows::core::{PCSTR, PSTR};
use windows::Win32::Foundation::{BOOL, HANDLE};
use windows::Win32::Security::SECURITY_ATTRIBUTES;
use windows::Win32::System::Threading::*;

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
    pub flags: Option<Vec<String>>,
    pub flags_append: bool,
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
            flags: None,
            flags_append: false,
            show_window: None,
            stdin_handle: None,
            stdout_handle: None,
            stderr_handle: None,
        }
    }
}

impl StartupInformation {
    pub fn as_native(&self) -> STARTUPINFOA {
        let mut si = STARTUPINFOA {
            cb: size_of::<STARTUPINFOA>() as u32,
            ..Default::default()
        };

        if self.desktop.is_some() {
            si.lpDesktop = PSTR(
                CString::new(self.desktop.to_owned().unwrap())
                    .unwrap()
                    .into_raw() as *mut u8,
            );
        }

        if self.title.is_some() {
            si.lpTitle = PSTR(
                CString::new(self.title.to_owned().unwrap())
                    .unwrap()
                    .into_raw() as *mut u8,
            );
        }

        if self.x.is_some() {
            si.dwX = self.x.unwrap()
        }

        if self.y.is_some() {
            si.dwY = self.y.unwrap()
        }

        if self.x_size.is_some() {
            si.dwXSize = self.x_size.unwrap()
        }

        if self.y_size.is_some() {
            si.dwYSize = self.y_size.unwrap()
        }

        if self.x_count_chars.is_some() {
            si.dwXCountChars = self.x_count_chars.unwrap()
        }

        if self.y_count_chars.is_some() {
            si.dwYCountChars = self.y_count_chars.unwrap()
        }

        if self.fill_attribute.is_some() {
            si.dwFillAttribute = self.fill_attribute.unwrap()
        }

        if self.flags.is_some() {
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

            if !self.flags_append {
                si.dwFlags.0 = 0;
            }

            for flag in self.flags.to_owned().unwrap() {
                if resolver.contains_key(flag.as_str()) {
                    si.dwFlags |= *resolver.get(flag.as_str()).unwrap()
                }
            }
        }

        if self.show_window.is_some() {
            si.wShowWindow = self.show_window.unwrap()
        }

        if self.stdin_handle.is_some() {
            si.hStdInput = HANDLE(self.stdin_handle.unwrap())
        }

        if self.stdout_handle.is_some() {
            si.hStdOutput = HANDLE(self.stdout_handle.unwrap())
        }

        if self.stderr_handle.is_some() {
            si.hStdError = HANDLE(self.stderr_handle.unwrap())
        }

        return si;
    }
}

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
    pub fn as_native(&self) -> SECURITY_ATTRIBUTES {
        let mut sa = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            ..Default::default()
        };

        if self.security_descriptor.is_some() {
            sa.lpSecurityDescriptor = self.security_descriptor.unwrap() as *mut c_void;
        }

        if self.inherit_handle.is_some() {
            sa.bInheritHandle = BOOL(if self.inherit_handle.unwrap() { 1 } else { 0 });
        }

        return sa;
    }
}

pub struct NativeCreationParams {
    pub process_attributes: SECURITY_ATTRIBUTES,
    pub thread_attributes: SECURITY_ATTRIBUTES,
    pub inherit_handles: BOOL,
    pub creation_flags: PROCESS_CREATION_FLAGS,
    pub environment: *const c_void,
    pub current_directory: PCSTR,
    pub startup_info: STARTUPINFOA,
}

impl Default for NativeCreationParams {
    fn default() -> NativeCreationParams {
        NativeCreationParams {
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

pub struct CreationParams {
    pub process_attributes: Option<SecurityAttributes>,
    pub thread_attributes: Option<SecurityAttributes>,
    pub inherit_handles: Option<bool>,
    pub creation_flags: Option<Vec<String>>,
    pub creation_flags_addmode: bool,
    pub environment: Option<String>,
    pub current_directory: Option<String>,
    pub startup_info: Option<StartupInformation>,
}

impl Default for CreationParams {
    fn default() -> CreationParams {
        CreationParams {
            process_attributes: None,
            thread_attributes: None,
            inherit_handles: None,
            creation_flags: None,
            creation_flags_addmode: false,
            environment: None,
            current_directory: None,
            startup_info: None,
        }
    }
}

impl CreationParams {
    pub fn as_native(&self) -> NativeCreationParams {
        let mut np = NativeCreationParams::default();

        if self.process_attributes.is_some() {
            np.process_attributes = self.process_attributes.unwrap().as_native();
        }

        if self.thread_attributes.is_some() {
            np.thread_attributes = self.thread_attributes.unwrap().as_native();
        }

        if self.inherit_handles.is_some() {
            np.inherit_handles = BOOL(if self.inherit_handles.unwrap() { 1 } else { 0 });
        }

        if self.creation_flags.is_some() {
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

            if !self.creation_flags_addmode {
                np.creation_flags.0 = 0;
            }

            for flag in self.creation_flags.unwrap() {
                if resolver.contains_key(flag.as_str()) {
                    np.creation_flags |= *resolver.get(flag.as_str()).unwrap()
                }
            }
        }

        // This is not correct
        if self.environment.is_some() {
            // np.environment = self.environment.unwrap().as_ptr() as *mut c_void;
        }

        if self.current_directory.is_some() {
            np.current_directory = PCSTR(
                CString::new(self.current_directory.unwrap())
                    .unwrap()
                    .into_raw() as *const u8,
            );
        }

        if self.startup_info.is_some() {
            np.startup_info = self.startup_info.unwrap().as_native();
        }

        return np;
    }
}

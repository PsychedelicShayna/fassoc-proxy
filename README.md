
# FASSOC Proxy - File Association Proxy (for Windows)
This is a file association proxy for Windows that provides an alternative way to associate certain types of files to certain programs, by associating them to this program instead, allowing this program to sit in the middle and act as the handler for those open requests, as opposed to the default handler that is used by "Open With" - why would you want to do this, what purpose does this serve? Well, consider the following benefits:

* The ability to open files with terminal commands and scripts, e.g. Python, PowerShell, Batch, etc.
* The ability to control what command line arguments are passed to associated programs.
* Complete control over the WinAPI process creation parameters.
* Being able to use RegEx to conditionally match files to different rules, based on file name and file content.
  * This can be used in combination with any of the above; you could conditionally associate two files of the same type to different programs, or to the same program, but providing different arguments depending on the file name or file content.
* Providing a blank slate to customize file type icons, without interfering with the registry keys of any existing programs.
  * A built-in way to change file type icons is planned, but in the meantime you can use [FileTypesMan](https://www.nirsoft.net/utils/file_types_manager.html), or simply edit the registry yourself.

Windows provides no facilities that make any of that possible out of the box, which is where this proxy handler comes in. By associating files with `fassoc-proxy.exe` instead of the target program, and then defining your own custom association rules in a JSON file, you gain the aforementioned control and increased flexibility that would not be possible otherwise.

## Contributing
Pull requests are open and appreciated! If you encounter any bugs, please [open an issue](https://github.com/PsychedelicShayna/fassoc-proxy/issues), and include the log file with your issue. If you can reproduce the error, then please do so using the debug build, so that the log file includes valuable debug messages, and if the invoked process prints anything notable to the console window when using the debug build, then please include that as well. 

### Priority Todo
These features in particular are the ones I'm hoping to target next, once I have the time. They're high up on my radar, as they're crucial for a good experience. 

* Taking complete control over the file type in the Windows registry.

  As of right now, overriding the default handler for a file type through a file's property menu so that it uses fassoc-proxy, does not actually give fassoc-proxy "ownership" of the file type, as it's considered a temporary user-defined "override" in the registry, rather than a proper handler definition.

  This will either set the icon to a generic gray executable fallback icon, or use a fallback icon defined in the registry. 

  This is very far from ideal, as you'd ruin icons for your file types, unless you manually head into the registry, in combination with a tool like FileTypesMan, to set the icons and file type definitions manually, which is far from user friendly and requires quite a bit of expertise in the way file types are defined and managed in the registry.

  * Once this is achieved, providing the user with the ability to select whatever icon they like for a file type should be trivial. Initially, pointing to the icon in the configuration should suffice. 
  * For convenience and quality of life, an override folder could eventually be implemented, where icon files matching the file extension will automatically be used as the icon for that file type.
  * Once the icons are out of the way, we could then focus on changing the file type's name and description as it's displayed in Windows Explorer, again, should be trivial if the parent bullet point has been accomplished.
  * Providing control over the file type's appearance in the "New >" submenu of the Windows Explorer directory context menu, to create new files of that type, as well as what name the file type appears as in that submenu.
* Porting the config format over to something more human friendly, such as YAML or TOML.
* Creating a CLI for fassoc-proxy in order to make the process of configuring it simpler. This would avoid the need to manually configure the configuration file. An interactive TUI is also an option.
* Employing some tamper protection features for the configuration file, so that a bad actor can't modify the configuration file such that it redirects to their application, by simply editing a file. 
  * For now, setting admin-only permissions on the configuration file, or better yet, both the file and the folder, independently, without inheritance, should be enough. 
  * In the future, perhaps the configuration file would need to have an accompanying signature that fassoc-proxy can safely validate in order for it to obey the configuration. This would have to be opt-in though, otherwise some might consider it a pain in the ass.

## Download
You can go ahead and download the latest build from the [releases]() section. It is recommended that you __do not place the binary in a location that requires administrator privileges to write to__ (such as Program Files), because the log file containing vital information about potential errors in your JSON configuration cannot be created there, and there currently is no fallback path, or UAC privilege request implemented yet. Instead, place it somewhere such as `AppData\Local\fassoc-proxy\`, or `C:\Tools\fassoc-proxy\`, etc. 

Each release contains a debug build as well as the main release build, with the primary difference (apart from containing debug symbols) being that the log level is lowered to debug, meaning the output to the log file will be much more verbose and contain useful debug information. A console window will also be created when using the debug build, that will show anything that the invoked process has printed to the standard output, as well as a copy of everything written to the log file.

## Configuration
Configuration is done through a JSON file, whose path is provided either through an environment variable (recommended), or through a command line argument (mainly used for debugging / trying out different rule files). The environment variable should be named `FASSOC_RULES_PATH`, and should point to the JSON file containing the rules. The name of this JSON file doesn't matter, but the convention is `fassoc-rules.json`

The configuration structure itself is divided into three JSON objects, called: mappings, matchers, and commands. The mappings object maps file extensions to a list of candidate matchers or commands, the matchers object attaches conditions to a command, and the commands object contains entries that represent invocations to programs.

---

* Commands: Entries in the commands object represent a command / program invocation - these are instructions on how to open a program, with what arguments, creation parameters, etc, and are decoupled from any specific extensions or conditions, they simply represent invocations.

* Matchers: Entries in the matchers object attach conditions to a command. They contain a command name, as well as optional values that represent conditions, for example: `"regexf"` which stores a RegEx pattern that the name of the file being opened must match, and `"regexc"`, another RegEx pattern that _content_ of the file being opened must match. If one or more conditions in a matchers entry fails, then the matcher and its associated command is ignored.

* Mappings: Entries in the mappings object map file extensions to a list of candidate matchers. This is how files are actually associated with commands, as the extension is looked up in the mappings object in order to get a list of matcher names, and the first matcher whose conditional checks all pass will get selected, and the command stored within the matcher will be used in order to open the file. The order in which matcher names appear in a mappings list matters, as each matcher is checked in the order they appear in the JSON, so if multiple matchers can match the same file, the one that appears first is selected, as it has the higher priority. 
  * Additionally, if a mapping list contains the name of a matcher that doesn't exist, but a command with that same name does exist, then it is interpreted as a command, and the command will be called directly without any additional condition checks, but if both a matcher and a command with the same name exist, the matcher will always receive priority.
  * If the file has no extension, or the extension simply isn't present in the mappings object, then the fallback mapping with the name `"*"` is used if it is defined. It is functionally the same as the other mappings; its value is a list of strings whose names correspond to matchers (or commands) and the first matcher that matches the file being opened will have its associated command used in order to open the file, whereas any commands encountered in the list will immediately be matched.

---

### Example Configuration
This is a simple example configuration that should highlight how the configuration file is structured. This example does not contain all available configuration keys, as most configuration keys are optional, and not necessary for a minimal example.

If the file being opened is a `txt` file, then the `"CMake"` matcher is checked first, as it appears first in the mapping list for `txt` files. The `"CMake"` matcher has a `"regexf"` condition, which means that its RegEx (in this case `CMakeLists`) must be contained within the file name in order for the condition check to pass, and for the `txt` file to be opened with the matcher's associated command: `"Neovim"` (which opens Neovim in Windows Terminal)

If the matcher's condition check fails, then the next matcher in the list is checked, but wait, `"Notepad"` isn't a matcher, but that's okay, because it's a command, and no matcher with the same name exists, so the `"Notepad"` entry will be treated as a command, and will be invoked directly with no additional conditions.

If any type of file other than a `txt` file is opened, then the fallback `"*"` mapping is used, which will unconditionally invoke the `"HXD"` command to open the file with the HXD hex editor, as no matcher with the name `"HXD"` exists, but a command with that name does exist.

---

```json
{
  "mappings": {
      "txt": [ "CMake", "Notepad" ],
      "*": [ "HXD" ]
  },

  "matchers": {
      "CMake": {
          "command": "Neovim",
          "regexf": "CMakeLists"
      }
  },

  "commands": {
      "Notepad": {
          "path": "C:\\Windows\\System32\\notepad.exe",
          "arguments": "~~$0 ~~$1"
      },

      "Neovim": {
          "path": "C:\\Program Files\\WindowsApps\\Microsoft.WindowsTerminal_1.14.2281.0_x64__8wekyb3d8bbwe\\wt.exe",
          "arguments": "~~$0 pwsh -c \"nvim ~~$1\""
      },

      "HXD": {
          "path": "C:\\Program Files (x86)\\HxD\\HxD.exe",
          "arguments": "~~$0 ~~$1"
      },
  }
}
```

---

### **Important!!**
* The `"path"` key must always be an absolute path to an executable, relative, non-canonical paths will be rejected (WinAPI limitation, might add a PATH variable resolver in the future).

* The `"path"` key also has to be an executable, it cannot be another file (this is to protect you from accidentally forkbombing yourself by pointing to a file that is registered to open with FASSOC Proxy, creating a recursive loop where FASSOC Proxy will keep launching itself forever).

* Notice how the arguments string for each command contains `~~$0` and `~~$1` - these refer to the command line arguments received by FASSOC Proxy when it was launched, e.g. `~~$N` where `N` is the argument index. By default, `~~$0` will always contain the path to FASSOC Proxy - **Windows requires that this always be included at the start of the arguments string, as the program will most likely crash without it** (I would have made it implicit, but figured more control is better than less), and `~~$1` will always contain the path to the file being opened if it was opened with FASSOC Proxy. 

* Command line argument substitution is available for the following strings, where `...` is the command name.
  * `commands/.../path`
  * `commands/.../arguments`
  * `commands/.../cwd`
  * `commands/.../extras/desktop`
  * `commands/.../extras/title`

## Complete Configuration Reference
Entries in the "commands" object have keys which are 1:1 WinAPI equivalents of the [CreateProcessA](https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessa) function. Naturally, not every argument makes sense to map into JSON (e.g. specifying the stdin/stdout/stderr handle in the [STARTUPINFOA](https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/ns-processthreadsapi-startupinfoa) struct) but everything that makes sense to map has been mapped. You can learn about what these options do by looking at the WinAPI documentation, as the JSON values will be fed directly into the call to `CreateProcessA`.

If a comment is formatted like this: `FunctionName(ArgumentName)` then the key/value is the JSON equivalent of an argument called `ArgumentName` belonging to the WinAPI function called `FunctionName`. If the comment is formatted like this: `TypeName::PropertyName` then the key/value is the JSON equivalent of a property called `PropertyName` belonging to the WinAPI type/struct called `TypeName`.

For JSON keys that are WinAPI equivalents, it is recommended that you read the relevant WinAPI documentation if you intend to use them.

The way bitmasks are formulated in JSON is via a list of strings representing different flags, with names that are 1:1 identical to what you would find in WinAPI. For example, a fill attribute of `FOREGROUND_RED | FOREGROUND_INTENSITY` in WinAPI would be represented as `["FOREGROUND_RED", "FOREGROUND_INTENSITY"]` in JSON, that is to say, all of the values that the strings represent are bitwise OR'd together. If the list contains a number, the number will be untouched, and simply OR'd with the rest of the string values.

```js
{
    "mappings": {
        // A mapping, where "txt" can be any file extension. The value being
        // a list of strings, that are either names of matchers, or names of
        // commands. If the name is that of a command, then the command is
        // invoked directly without any checks. If there is both a matcher
        // and a command by the same name, the matcher takes priority.
        // If the name is that of a matcher, then the command that is
        // associated with the matcher is invoked if the matcher's checks
        // pass, and if it fails, then the matcher is skipped and the next
        // matcher is checked. Each matcher is checked in the order they appear.
        "txt": [ "name of matcher or command" ],

        // Fallback mapping, used if there is no other applicable mapping.
        "*": [ "TestMatcher", "TestCommand" ]
    },

    "matchers": {
        "TestMatcher": {
            // The command associated with this matcher, that will get invoked
            // if the matcher checks pass. This is the only mandatory key for
            // a matcher, all of the condition checking keys are optional. If
            // multiple condition keys are defined, then the conditions are
            // AND-ed together, meaning that all of the conditions must pass
            "command": "TestCommand",

            // A RegEx pattern condition that matches against the name of the file being opened.
            "regexf": "<regex string>",

            // A RegEx pattern condition that matches against the contents of the file being opened.
            "regexc": "<regex string>"
        }
    },

    "commands": {
        "TestCommand": {
            // CreateProcessA(lpApplicationName): 
            // The absolute path to the executable.
            "path": "C:\\Windows\\System32\\cmd.exe",

            // CreateProcessA(lpCommandLine)
            // The command line argument string to pass to the program.
            "arguments": "~~$0 ~~$1",

            // CreateProcessA(lpCurrentDirectory)
            // The working directory to use when launching the program.
            "cwd": "~~$1\\..",
            
            // CreateProcessA(bInheritHandles)
            // If true, each inheritable handle in the calling process is inherited by 
            // the new process. If false, the handles are not inherited
            "inherit_handles": true,
            
            // CreateProcessA(dwCreationFlags) (bitmask)
            // The flags that control the priority class and the creation of the process. 
            // Values: https://docs.microsoft.com/en-us/windows/win32/procthread/process-creation-flags
            "creation_flags": [
                "CREATE_BREAKAWAY_FROM_JOB",
                "CREATE_DEFAULT_ERROR_MODE",
                "CREATE_NEW_CONSOLE",
                "CREATE_NEW_PROCESS_GROUP",
                "CREATE_NO_WINDOW",
                "CREATE_PROTECTED_PROCESS",
                "CREATE_PRESERVE_CODE_AUTHZ_LEVEL",
                "CREATE_SECURE_PROCESS",
                "CREATE_SEPARATE_WOW_VDM",
                "CREATE_SHARED_WOW_VDM",
                "CREATE_SUSPENDED",
                "CREATE_UNICODE_ENVIRONMENT",
                "DEBUG_ONLY_THIS_PROCESS",
                "DEBUG_PROCESS",
                "DETACHED_PROCESS",
                "EXTENDED_STARTUPINFO_PRESENT",
                "INHERIT_PARENT_AFFINITY",
            ],

            // CreateProcessA(lpStartupInfo)
            // A STARTUPINFOA instance that defines additional startup properties for the process.
            // Please view the STARTUPINFOA documentation to understand what these values do.
            // STARTUPINFOA Documentation: https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/ns-processthreadsapi-startupinfoa
            "extras": {
                // STARTUPINFOA::lpDesktop
                "desktop": "",

                // STARTUPINFOA::lpTitle
                "title": "",
  
                // STARTUPINFOA::dwX
                "x": 0,

                // STARTUPINFOA::dwY
                "y": 0,

                // STARTUPINFOA::dwXSize
                "x_size": 0,

                // STARTUPINFOA::dwYSize
                "y_size": 0,

                // STARTUPINFOA::dwXCountChars
                "x_count_chars": 0,

                // STARTUPINFOA::dwYCountChars
                "y_count_chars": 0,

                // STARTUPINFOA::dwFillAttribute (bitmask)
                "fill_attribute": [
                    "FOREGROUND_BLUE",
                    "FOREGROUND_RED",
                    "FOREGROUND_GREEN",
                    "BACKGROUND_BLUE",
                    "BACKGROUND_RED",
                    "BACKGROUND_GREEN",
                    "BACKGROUND_INTENSITY",
                    "FOREGROUND_INTENSITY",
                    "COMMON_LVB_LEADING_BYTE",
                    "COMMON_LVB_TRAILING_BYT",
                    "COMMON_LVB_GRID_HORIZONTAL",
                    "COMMON_LVB_GRID_LVERTICAL",
                    "COMMON_LVB_GRID_RVERTICAL",
                    "COMMON_LVB_REVERSE_VIDEO",
                    "COMMON_LVB_UNDERSCORE",
                    "COMMON_LVB_SBCSDBCS"
                ],
                
                // STARTUPINFOA::dwFlags (bitmask)
                "flags": [
                    "STARTF_FORCEONFEEDBACK",
                    "STARTF_FORCEOFFFEEDBACK",
                    "STARTF_PREVENTPINNING",
                    "STARTF_RUNFULLSCREEN",
                    "STARTF_TITLEISAPPID",
                    "STARTF_TITLEISLINKNAME",
                    "STARTF_UNTRUSTEDSOURCE",
                    "STARTF_USECOUNTCHARS",
                    "STARTF_USEFILLATTRIBUTE",
                    "STARTF_USEHOTKEY",
                    "STARTF_USEPOSITION",
                    "STARTF_USESHOWWINDOW",
                    "STARTF_USESIZE",
                    "STARTF_USESTDHANDLES"
                ]
            }
        }
    }
}

```





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

## Download
You can go ahead and download the latest build from the [releases]() section. It is recommended that you __do not place the binary in a location that requires administrator privileges to write to__ (such as Program Files), because the log file containing vital information about potential errors in your JSON configuration cannot be created there, and there currently is no fallback path, or UAC privilege request implemented yet. Instead, place it somewhere such as `AppData\Local\fassoc-proxy\`, or `C:\Tools\fassoc-proxy\`, etc.

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

* Command line argument substitution is available for the following strings:
  * "commands/.../path"
  * "commands/.../arguments"
  * "commands/.../cwd"
  * "extras/desktop"
  * "extras/title"

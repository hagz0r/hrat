use std::sync::Arc;

use async_trait::async_trait;
use image::codecs::jpeg::JpegEncoder;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

use crate::actors::{Actor, Command, HandlerResult, WsMessageSender};
use scap::capturer::{Capturer, Options};
use scap::frame::{convert_bgra_to_rgb, Frame};

type StreamingState = Arc<Mutex<bool>>;

const STREAM_PREFIX: u8 = 0x02;

pub struct RemoteScreen {
    capturer: Arc<Mutex<Capturer>>,
    is_streaming: StreamingState,
}

#[async_trait]
impl Actor for RemoteScreen {
    fn new() -> Self {
        let capturer = Self::get_capturer().expect("Failed to initialize capturer");

        Self {
            is_streaming: Arc::new(Mutex::new(false)),
            capturer: Arc::new(Mutex::new(capturer)),
        }
    }

    async fn handler(&mut self, args: Command, writer: WsMessageSender) -> HandlerResult {
        let action = args["action"].as_str().unwrap_or("");

        match action {
            "screenshot" => {
                let should_compress = args["compressing"].as_bool().unwrap_or(true);
                match Self::get_photo(self.capturer.clone(), should_compress).await {
                    Ok(image_data) => {
                        let mut prefixed_data = vec![STREAM_PREFIX];
                        prefixed_data.extend(image_data);
                        writer.send(Message::Binary(prefixed_data)).await?;
                        crate::dev_print!("Photo sent by rem screen actor.");
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            "stream_start" => {
                let mut stream_guard = self.is_streaming.lock().await;
                if *stream_guard {
                    crate::dev_print!("Stream is already running.");
                    return Ok(());
                }
                *stream_guard = true;
                crate::dev_print!("Starting desktop stream...");

                let stream_state = self.is_streaming.clone();
                let stream_writer = writer.clone();
                let capturer_clone = self.capturer.clone();

                tokio::spawn(Self::run_streaming_loop(
                    capturer_clone,
                    stream_state,
                    stream_writer,
                ));
                Ok(())
            }
            "stream_stop" => {
                let mut stream_guard = self.is_streaming.lock().await;
                *stream_guard = false;
                crate::dev_print!("Stopping desktop stream...");
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Unknown desktop action '{}'", action)),
        }
    }
}

impl RemoteScreen {
    async fn get_photo(
        capturer: Arc<Mutex<Capturer>>,
        should_compress: bool,
    ) -> anyhow::Result<Vec<u8>> {
        let (frame, width, height) = {
            let mut capturer_guard = capturer.lock().await;
            let frame = capturer_guard.get_next_frame()?;
            let [width, height] = capturer_guard.get_output_frame_size();
            (frame, width, height)
        };
        Self::encode_frame(frame, width, height, should_compress)
    }

    // FUCK IMAGE ENCODING
    fn encode_frame(
        frame: Frame,
        width: u32,
        height: u32,
        compress: bool,
    ) -> anyhow::Result<Vec<u8>> {
        let rgb: Vec<u8> = match frame {
            Frame::BGRA(f) => convert_bgra_to_rgb(f.data), // BGRA → RGB
            Frame::BGRx(f) => {
                // BGRx to RGB
                let mut out = Vec::with_capacity(f.data.len() / 4 * 3);
                for px in f.data.chunks_exact(4) {
                    out.extend_from_slice(&[px[2], px[1], px[0]]); // B,G,R → R,G,B
                }
                out
            }
            Frame::BGR0(f) => {
                // BGR0 to RGB
                let mut out = Vec::with_capacity(f.data.len() / 4 * 3);
                for px in f.data.chunks_exact(4) {
                    out.extend_from_slice(&[px[2], px[1], px[0]]); // B,G,R → R,G,B
                }
                out
            }
            Frame::RGBx(f) => {
                // RGBx to RGB
                let mut out = Vec::with_capacity(f.data.len() / 4 * 3);
                for px in f.data.chunks_exact(4) {
                    out.extend_from_slice(&[px[0], px[1], px[2]]); // already RGB, just drop X
                }
                out
            }
            Frame::RGB(f) => {
                // RGB to RGB
                let mut out = Vec::with_capacity(f.data.len() / 3 * 3);
                for px in f.data.chunks_exact(3) {
                    out.extend_from_slice(&[px[0], px[1], px[2]]); // already RGB
                }
                out
            }
            other => return Err(anyhow::anyhow!("Unsupported frame: {:?}", other)),
        };

        if compress {
            let mut jpeg = Vec::new();
            let mut enc = JpegEncoder::new_with_quality(&mut jpeg, 75);
            enc.encode(&rgb, width, height, image::ExtendedColorType::Rgb8)?;
            Ok(jpeg)
        } else {
            Ok(rgb)
        }
    }

    async fn run_streaming_loop(
        capturer: Arc<Mutex<Capturer>>,
        is_streaming: StreamingState,
        writer: WsMessageSender,
    ) {
        crate::dev_print!("[RS Stream] Loop started.");
        {
            let mut capturer_guard = capturer.lock().await;
            capturer_guard.start_capture();
        }
        crate::dev_print!("[RS Stream] Capturer started. Entering capture loop...");

        loop {
            if !*is_streaming.lock().await {
                crate::dev_print!("[RS Stream] Streaming flag is false, stopping loop.");
                break;
            }

            // --- THIS IS THE FINAL, CORRECT IMPLEMENTATION ---
            let capturer_clone = capturer.clone();
            let blocking_task = tokio::task::spawn_blocking(move || {
                // This code now runs on a separate, blocking-safe thread.
                let mut capturer_guard = capturer_clone.blocking_lock();
                let frame_result = capturer_guard.get_next_frame();
                let [w, h] = capturer_guard.get_output_frame_size();
                (frame_result, w, h)
            });

            match tokio::time::timeout(std::time::Duration::from_secs(2), blocking_task).await {
                // Timeout completed, and the blocking task didn't panic
                Ok(Ok((Ok(frame), width, height))) => {
                    if width == 0 || height == 0 {
                        continue;
                    }

                    match Self::encode_frame(frame, width, height, true) {
                        Ok(encoded_frame) => {
                            let mut prefixed_frame = vec![STREAM_PREFIX];
                            prefixed_frame.extend(encoded_frame);
                            if writer.send(Message::Binary(prefixed_frame)).await.is_err() {
                                crate::dev_print!(
                                    "[RS Stream] ERROR: Failed to send frame, stopping."
                                );
                                break;
                            }
                        }
                        Err(e) => {
                            crate::dev_print!("[RS Stream] ERROR: Frame encode failed: {}", e)
                        }
                    }
                }
                // Timeout completed, blocking task didn't panic, but frame capture returned an error
                Ok(Ok((Err(e), _, _))) => {
                    crate::dev_print!("[RS Stream] ERROR: Frame capture failed internally: {}", e);
                }
                // Timeout completed, but the blocking task panicked (crashed)
                Ok(Err(e)) => {
                    crate::dev_print!("[RS Stream] FATAL: Blocking task panicked: {}", e);
                }
                // The timeout of 2 seconds elapsed.
                Err(_) => {
                    crate::dev_print!("[RS Stream] FATAL: Frame capture timed out. The 'scap' library is hung. This is a known issue on Wayland display servers. Please try logging into an X11/X.org session and try again.");
                }
            }

            // Limit FPS
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        {
            let mut capturer_guard = capturer.lock().await;
            capturer_guard.stop_capture();
        }
        crate::dev_print!("[RS Stream] Loop finished and capturer stopped.");
    }
    fn get_capturer() -> anyhow::Result<Capturer> {
        if !scap::is_supported() {
            return Err(anyhow::anyhow!("Scap is not supported"));
        }

        if !scap::has_permission() {
            if !scap::request_permission() {
                return Err(anyhow::anyhow!(
                    "Scap does not have a permission to operate"
                ));
            }
        }

        let primary_target = None;

        let options = Options {
            fps: 30,
            target: primary_target,
            show_cursor: true,
            show_highlight: true,
            excluded_targets: None,
            output_type: scap::frame::FrameType::BGRAFrame,
            output_resolution: scap::capturer::Resolution::_720p,
            crop_area: None,
        };

        Capturer::build(options).map_err(|err| err.into())
    }
}

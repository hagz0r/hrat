use std::sync::Arc;

use async_trait::async_trait;
use tokio::{sync::Mutex, task};
use tokio_tungstenite::tungstenite::Message;

use crate::actors::{Actor, Command, HandlerResult, WsMessageSender};
use scap::{
    capturer::{Area, Capturer, Options, Point, Size},
    *,
};

type StreamingState = Arc<Mutex<bool>>;

pub struct RemoteScreen {
    capturer: Option<Arc<Mutex<Capturer>>>,
    is_streaming: StreamingState,
}

#[async_trait]
impl Actor for RemoteScreen {
    fn new() -> Self {
        Self {
            is_streaming: Arc::new(Mutex::new(false)),
            capturer: None,
        }
    }

    async fn handler(&mut self, args: Command, writer: WsMessageSender) -> HandlerResult {
        self.capturer = Some(Arc::new(Mutex::new(Self::get_capturer()?)));

        let mode = args["mode"].as_str().unwrap_or("");

        match mode {
            "photo" => {
                let should_compress = args["compressing"].as_bool().unwrap_or(true);
                match Self::get_photo(should_compress).await {
                    Ok(image_data) => {
                        writer.send(Message::Binary(image_data)).await?;
                        crate::dev_print!("Photo sent by rem screen actor.");
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
                crate::dev_print!("Starting desktop stream...");

                let stream_state = self.is_streaming.clone();
                let stream_writer = writer.clone();

                tokio::spawn(Self::run_streaming_loop(stream_state, stream_writer));
                Ok(())
            }
            "video_stop" => {
                let mut stream_guard = self.is_streaming.lock().await;
                *stream_guard = false;
                crate::dev_print!("Stopping desktop stream...");
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Unknown desktop mode '{}'", mode)),
        }
    }
}

impl RemoteScreen {
    async fn get_photo(should_compress: bool) -> anyhow::Result<Vec<u8>> {
        // task::spawn_blocking(move || {
        //     let frame = self
        //         .capturer
        //         .as_ref()
        //         .unwrap()
        //         .blocking_lock()
        //         .get_next_frame()
        //         .map_err(|e| anyhow::anyhow!(e))
        //         .unwrap();

        //     if should_compress {
        //         // Ok(frame.into())
        //     } else {
        //         // Ok(frame.into())
        //     }
        // })
        // .await
        // .unwrap();

        Ok(vec![])
    }

    async fn run_streaming_loop(_is_streaming: StreamingState, _writer: WsMessageSender) {}

    fn get_capturer() -> anyhow::Result<Capturer> {
        if !scap::is_supported() {
            return Err(anyhow::anyhow!("Scap is not supported"));
        }

        if scap::has_permission() {
            if !scap::request_permission() {
                return Err(anyhow::anyhow!(
                    "Scap does not have a permission to operate"
                ));
            }
        }

        let targets = scap::get_all_targets();
        println!("Targets: {:?}", targets);

        // All your displays and windows are targets
        // You can filter this and capture the one you need.

        // Create Options
        let options = Options {
            fps: 60,
            target: None, // None captures the primary display
            show_cursor: true,
            show_highlight: true,
            excluded_targets: None,
            output_type: scap::frame::FrameType::BGRAFrame,
            output_resolution: scap::capturer::Resolution::_720p,
            crop_area: Some(Area {
                origin: Point { x: 0.0, y: 0.0 },
                size: Size {
                    width: 2000.0,
                    height: 1000.0,
                },
            }),
            // ..Default::default()
        };

        Capturer::build(options).map_err(|err| err.into())
    }
}

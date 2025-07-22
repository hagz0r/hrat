use async_trait::async_trait;
use image::codecs::jpeg::JpegEncoder;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

use crate::actors::{Actor, Command, HandlerResult, WsMessageSender};
use scap::capturer::{Capturer, Options, Resolution};
use scap::frame::{convert_bgra_to_rgb, Frame, FrameType};

type StreamingState = Arc<Mutex<bool>>;
const STREAM_PREFIX: u8 = 0x02;

/*
*  I literraly hated using this lib, as it's buggy and shitty
*  I've spent around 6 hours to debug and fix this shit to make it work
   Haven't tested on Windows yet
*
*
*
*/

#[derive(Clone, Copy, Debug)]
pub struct EncodeParams {
    pub jpeg_quality: u8,
    pub frame_interval: Duration,
    pub first_frame_timeout: Duration,
    pub first_frame_poll: Duration,
}

impl Default for EncodeParams {
    fn default() -> Self {
        Self {
            jpeg_quality: 60,
            frame_interval: Duration::from_millis(100),
            first_frame_timeout: Duration::from_millis(750),
            first_frame_poll: Duration::from_millis(10),
        }
    }
}

pub struct RemoteScreen {
    capturer: Arc<Mutex<Capturer>>,
    is_streaming: StreamingState,
    params: EncodeParams,
}

#[async_trait]
impl Actor for RemoteScreen {
    fn new() -> Self {
        crate::dev_print!("[LOG] RemoteScreen::new() called.");
        let capturer = Self::build_capturer().expect("Failed to initialize capturer");
        crate::dev_print!("[LOG] RemoteScreen::new() capturer initialized successfully.");

        Self {
            is_streaming: Arc::new(Mutex::new(false)),
            capturer: Arc::new(Mutex::new(capturer)),
            params: EncodeParams::default(),
        }
    }

    async fn handler(&mut self, args: Command, writer: WsMessageSender) -> HandlerResult {
        let action = args["action"].as_str().unwrap_or("");
        crate::dev_print!("[LOG] RemoteScreen::handler action: '{action}'");

        match action {
            "screenshot" => {
                let should_compress = args["compressing"].as_bool().unwrap_or(true);
                let data =
                    Self::grab_single_frame(self.capturer.clone(), should_compress, self.params)
                        .await?;
                let mut buf = Vec::with_capacity(1 + data.len());
                buf.push(STREAM_PREFIX);
                buf.extend(data);
                writer.send(Message::Binary(buf)).await?;
                Ok(())
            }
            "stream_start" => {
                let mut guard = self.is_streaming.lock().await;
                if *guard {
                    return Ok(());
                }
                *guard = true;
                let state = self.is_streaming.clone();
                let capt = self.capturer.clone();
                let params = self.params;
                let wr = writer.clone();
                tokio::spawn(async move {
                    if let Err(e) = Self::stream_task(capt, state, wr, params).await {
                        crate::dev_print!("[RS Stream] error {e}");
                    }
                });
                Ok(())
            }
            "stream_stop" => {
                *self.is_streaming.lock().await = false;
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Unknown desktop action '{action}'")),
        }
    }
}

impl RemoteScreen {
    // ---------------- single‑frame (screenshot) --------------------------------------------------
    async fn grab_single_frame(
        capturer: Arc<Mutex<Capturer>>,
        compress: bool,
        params: EncodeParams,
    ) -> anyhow::Result<Vec<u8>> {
        let mut cap = capturer.lock().await;
        cap.start_capture();
        let deadline = Instant::now() + params.first_frame_timeout;
        let frame = loop {
            match cap.get_next_frame() {
                Ok(f) => break f,
                Err(e) if Self::is_would_block(&e) && Instant::now() < deadline => {
                    std::thread::sleep(params.first_frame_poll);
                }
                Err(e) => {
                    cap.stop_capture();
                    return Err(e.into());
                }
            }
        };
        let (w, h) = Self::frame_dims(&frame);
        let out = Self::encode_frame(&frame, w, h, compress, params.jpeg_quality)?;
        cap.stop_capture();
        Ok(out)
    }

    // ---------------- continuous stream ---------------------------------------------------------
    async fn stream_task(
        capturer: Arc<Mutex<Capturer>>,
        is_streaming: StreamingState,
        writer: WsMessageSender,
        params: EncodeParams,
    ) -> anyhow::Result<()> {
        // -- init & first frame --
        {
            let mut cap = capturer.lock().await;
            cap.start_capture();
            let deadline = Instant::now() + params.first_frame_timeout;
            let first = loop {
                match cap.get_next_frame() {
                    Ok(f) => break f,
                    Err(e) if Self::is_would_block(&e) && Instant::now() < deadline => {
                        std::thread::sleep(params.first_frame_poll);
                    }
                    Err(e) => {
                        cap.stop_capture();
                        return Err(e.into());
                    }
                }
            };
            let (w, h) = Self::frame_dims(&first);
            let enc = Self::encode_frame(&first, w, h, true, params.jpeg_quality)?;
            let mut out = Vec::with_capacity(1 + enc.len());
            out.push(STREAM_PREFIX);
            out.extend(enc);
            writer.send(Message::Binary(out)).await?;
        }

        while *is_streaming.lock().await {
            let capt = capturer.clone();
            let q = params.jpeg_quality;
            let res = tokio::task::spawn_blocking(move || {
                let cap = capt.blocking_lock();
                match cap.get_next_frame() {
                    Ok(frm) => {
                        let (w, h) = RemoteScreen::frame_dims(&frm);
                        RemoteScreen::encode_frame(&frm, w, h, true, q)
                    }
                    Err(e) => Err(e.into()),
                }
            })
            .await?;

            if let Ok(buf) = res {
                let mut out = Vec::with_capacity(1 + buf.len());
                out.push(STREAM_PREFIX);
                out.extend(buf);
                if writer.send(Message::Binary(out)).await.is_err() {
                    break;
                }
            }
            tokio::time::sleep(params.frame_interval).await;
        }
        // stop
        capturer.lock().await.stop_capture();
        Ok(())
    }

    fn is_would_block<E: std::fmt::Display>(e: &E) -> bool {
        let s = e.to_string().to_ascii_lowercase();
        s.contains("would") || s.contains("again") || s.contains("wait") || s.contains("no frame")
    }

    fn frame_dims(frame: &Frame) -> (u32, u32) {
        match frame {
            Frame::BGRA(f) => (f.width as u32, f.height as u32),
            Frame::BGRx(f) => (f.width as u32, f.height as u32),
            Frame::BGR0(f) => (f.width as u32, f.height as u32),
            Frame::RGBx(f) => (f.width as u32, f.height as u32),
            Frame::RGB(f) => (f.width as u32, f.height as u32),
            _ => (0, 0),
        }
    }

    fn encode_frame(
        frame: &Frame,
        width: u32,
        height: u32,
        compress: bool,
        jpeg_quality: u8,
    ) -> anyhow::Result<Vec<u8>> {
        let rgb: Vec<u8> = match frame {
            Frame::BGRA(f) => convert_bgra_to_rgb(f.data.clone()),
            Frame::BGRx(f) => {
                let mut out = Vec::with_capacity((f.data.len() / 4) * 3);
                for px in f.data.chunks_exact(4) {
                    out.extend_from_slice(&[px[2], px[1], px[0]]);
                }
                out
            }
            Frame::BGR0(f) => {
                let mut out = Vec::with_capacity((f.data.len() / 4) * 3);
                for px in f.data.chunks_exact(4) {
                    out.extend_from_slice(&[px[2], px[1], px[0]]);
                }
                out
            }
            Frame::RGBx(f) => {
                let mut out = Vec::with_capacity((f.data.len() / 4) * 3);
                for px in f.data.chunks_exact(4) {
                    out.extend_from_slice(&[px[0], px[1], px[2]]);
                }
                out
            }
            Frame::RGB(f) => {
                let mut out = Vec::with_capacity(f.data.len());
                out.extend_from_slice(&f.data);
                out
            }
            _ => return Err(anyhow::anyhow!("Unsupported frame format")),
        };

        if !compress {
            return Ok(rgb);
        }
        let mut jpeg = Vec::new();
        JpegEncoder::new_with_quality(&mut jpeg, jpeg_quality).encode(
            &rgb,
            width,
            height,
            image::ExtendedColorType::Rgb8,
        )?;
        Ok(jpeg)
    }

    fn build_capturer() -> anyhow::Result<Capturer> {
        if !scap::is_supported() {
            return Err(anyhow::anyhow!("scap not supported"));
        }
        if !scap::has_permission() && !scap::request_permission() {
            return Err(anyhow::anyhow!("scap permission denied"));
        }
        let opts = Options {
            fps: 30,
            target: None,
            show_cursor: true,
            show_highlight: true,
            excluded_targets: None,
            output_type: FrameType::BGRAFrame,
            output_resolution: Resolution::_1080p,
            crop_area: None,
        };
        Capturer::build(opts).map_err(|e| e.into())
    }
}

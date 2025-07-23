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

/* ───────────────── timimg params ───────────────────────── */

#[derive(Clone, Copy)]
struct EncodeParams {
    jpeg_quality: u8,
    frame_interval: Duration,
    first_frame_timeout: Duration,
    first_frame_poll: Duration,
}

impl EncodeParams {
    fn new(fps: u32, jpeg_quality: u8) -> Self {
        Self {
            jpeg_quality,
            frame_interval: Duration::from_millis((1000 / fps.max(1)) as u64),
            first_frame_timeout: Duration::from_millis(500),
            first_frame_poll: Duration::from_millis(5),
        }
    }
}
impl Default for EncodeParams {
    fn default() -> Self {
        Self::new(30, 60)
    }
}

/* ───────────────── windows Send‑shim ───────────────────── */

#[cfg(target_os = "windows")]
mod win_send {
    use scap::capturer::Capturer;
    use std::ops::{Deref, DerefMut};

    pub struct SendCapturer(pub Capturer);
    unsafe impl Send for SendCapturer {}
    unsafe impl Sync for SendCapturer {}

    impl Deref for SendCapturer {
        type Target = Capturer;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl DerefMut for SendCapturer {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
}
#[cfg(target_os = "windows")]
type CapInner = win_send::SendCapturer;
#[cfg(not(target_os = "windows"))]
type CapInner = scap::capturer::Capturer;

#[cfg(target_os = "windows")]
fn into_cap_inner(c: Capturer) -> CapInner {
    win_send::SendCapturer(c)
}
#[cfg(not(target_os = "windows"))]
fn into_cap_inner(c: Capturer) -> CapInner {
    c
}

/* ───────────────── actor struct ────────────────────────── */

pub struct RemoteScreen {
    capturer: Arc<Mutex<CapInner>>,
    is_streaming: StreamingState,
    params: EncodeParams,
}
#[cfg(target_os = "windows")]
unsafe impl Send for RemoteScreen {}

#[async_trait]
impl Actor for RemoteScreen {
    fn new() -> Self {
        let cap = build_capturer().expect("capturer init failed");
        Self {
            capturer: Arc::new(Mutex::new(cap)),
            is_streaming: Arc::new(Mutex::new(false)),
            params: EncodeParams::default(),
        }
    }

    async fn handler(&mut self, args: Command, tx: WsMessageSender) -> HandlerResult {
        match args["action"].as_str().unwrap_or("") {
            "screenshot" => {
                let compress = args["compressing"].as_bool().unwrap_or(true);
                let img =
                    Self::grab_single_frame(self.capturer.clone(), compress, self.params).await?;
                let mut buf = Vec::with_capacity(1 + img.len());
                buf.push(STREAM_PREFIX);
                buf.extend(img);
                tx.send(Message::Binary(buf)).await?;
            }
            "stream_start" => {
                let mut flag = self.is_streaming.lock().await;
                if *flag {
                    return Ok(());
                }
                *flag = true;
                let cap = self.capturer.clone();
                let state = self.is_streaming.clone();
                let p = self.params;
                let w = tx.clone();
                tokio::spawn(async move {
                    let _ = stream_task(cap, state, w, p).await;
                });
            }
            "stream_stop" => *self.is_streaming.lock().await = false,
            a => return Err(anyhow::anyhow!("unknown action {a}")),
        }
        Ok(())
    }
}

/* ───────────────── impl details ────────────────────────── */
impl RemoteScreen {
    /* ---- single screenshot ---- */
    async fn grab_single_frame(
        capturer: Arc<Mutex<CapInner>>,
        compress: bool,
        p: EncodeParams,
    ) -> anyhow::Result<Vec<u8>> {
        let mut cap = capturer.lock().await;
        cap.start_capture();
        let deadline = Instant::now() + p.first_frame_timeout;

        let frame = loop {
            match cap.get_next_frame() {
                Ok(f) => break f,
                Err(e) if is_would_block(&e) && Instant::now() < deadline => {
                    drop(cap);
                    tokio::time::sleep(p.first_frame_poll).await;
                    cap = capturer.lock().await;
                }
                Err(e) => {
                    cap.stop_capture();
                    return Err(e.into());
                }
            }
        };
        cap.stop_capture();
        drop(cap);

        let (w, h) = get_frame_dimensions(&frame);
        encode_frame(&frame, w as u32, h as u32, compress, p.jpeg_quality)
    }
}

/* ---- continuous stream ---- */
async fn stream_task(
    capturer: Arc<Mutex<CapInner>>,
    state: StreamingState,
    tx: WsMessageSender,
    p: EncodeParams,
) -> anyhow::Result<()> {
    {
        capturer.lock().await.start_capture();
    }

    while *state.lock().await {
        let frame = {
            let cap = capturer.lock().await;
            match cap.get_next_frame() {
                Ok(f) => f,
                Err(e) if is_would_block(&e) => continue,
                Err(e) => return Err(e.into()),
            }
        };
        let (w, h) = get_frame_dimensions(&frame);
        let img = encode_frame(&frame, w as u32, h as u32, true, p.jpeg_quality)?;

        let mut out = Vec::with_capacity(1 + img.len());
        out.push(STREAM_PREFIX);
        out.extend(img);
        if tx.send(Message::Binary(out)).await.is_err() {
            break;
        }

        tokio::time::sleep(p.frame_interval).await;
    }
    capturer.lock().await.stop_capture();
    Ok(())
}

/* ---- helpers ---- */
fn encode_frame(
    frame: &Frame,
    width: u32,
    height: u32,
    compress: bool,
    q: u8,
) -> anyhow::Result<Vec<u8>> {
    let rgb = match frame {
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
        Frame::RGB(f) => f.data.clone(),
        _ => return Err(anyhow::anyhow!("unsupported frame fmt")),
    };
    if !compress {
        return Ok(rgb);
    }

    let mut jpeg = Vec::with_capacity(rgb.len() / 10);
    JpegEncoder::new_with_quality(&mut jpeg, q).encode(
        &rgb,
        width,
        height,
        image::ExtendedColorType::Rgb8,
    )?;
    Ok(jpeg)
}

fn is_would_block<E: std::fmt::Display>(e: &E) -> bool {
    let s = e.to_string().to_ascii_lowercase();
    s.contains("would") || s.contains("again") || s.contains("wait") || s.contains("no frame")
}

fn get_frame_dimensions(frame: &Frame) -> (usize, usize) {
    match frame {
        Frame::BGRA(f) => (f.width as usize, f.height as usize),
        Frame::RGB(f) => (f.width as usize, f.height as usize),
        Frame::RGBx(f) => (f.width as usize, f.height as usize),
        Frame::XBGR(f) => (f.width as usize, f.height as usize),
        Frame::YUVFrame(f) => (f.width as usize, f.height as usize),
        Frame::BGRx(f) => (f.width as usize, f.height as usize),
        Frame::BGR0(f) => (f.width as usize, f.height as usize),
    }
}

fn build_capturer() -> anyhow::Result<CapInner> {
    if !scap::is_supported() {
        anyhow::bail!("scap unsupported");
    }
    if !scap::has_permission() && !scap::request_permission() {
        anyhow::bail!("no permission");
    }

    let opts = Options {
        fps: 30,
        target: None,
        show_cursor: true,
        show_highlight: false,
        excluded_targets: None,
        output_type: FrameType::BGRAFrame,
        output_resolution: Resolution::_1080p,
        crop_area: None,
    };
    Ok(into_cap_inner(Capturer::build(opts)?))
}

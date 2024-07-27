use tokio::sync::mpsc;
// use webrtc::media::rtp::rtp_receiver::RTCRtpReceiver;
// use webrtc::media::rtp::rtp_codec::RTPCodecType;
// use webrtc::media::track::track_remote::TrackRemote;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use tungstenite::protocol::WebSocket;
use tokio_tungstenite::MaybeTlsStream;
use tokio::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tungstenite::Message;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::rtp_transceiver::rtp_receiver::RTCRtpReceiver;
use webrtc::track::track_remote::TrackRemote;

pub async fn handle_remote_screen(
	payload: &[u8],
	socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
) -> Result<(), Box<dyn std::error::Error>> {
	let mut media_engine = MediaEngine::default();

	media_engine.register_default_codecs()?;

	let api = APIBuilder::new().with_media_engine(media_engine).build();
	let config = RTCConfiguration::default();
	let peer_connection = Arc::new(api.new_peer_connection(config).await?);

	let (tx, mut rx) = mpsc::channel::<Arc<TrackRemote>>(1);

	peer_connection.on_track(Box::new(move |track: Option<Arc<TrackRemote>>, _: Option<Arc<RTCRtpReceiver>>| {
		if let Some(track) = track {
			let _ = tx.try_send(track);
		}
		Box::pin(async {})
	}));

	let offer = RTCSessionDescription::offer(String::from_utf8(payload.to_vec())?)?;
	peer_connection.set_remote_description(offer).await?;

	let answer = peer_connection.create_answer(None).await?;
	peer_connection.set_local_description(answer).await?;

	tokio::spawn(async move {
		while let Some(track) = rx.recv().await {
			let codec = track.codec();
			if codec.capability.mime_type == "video/vp8" || codec.capability.mime_type == "video/h264" {
				let mut rtp_packet: Vec<u8> = vec![0; 1600];
				while let Ok((n, _)) = track.read(&mut rtp_packet).await {
					let i = n.payload.len();
					socket.write(Message::Binary(rtp_packet[..i].to_vec())).unwrap();
					
				}
			}
		}
	});

	// Wait for the peer connection to be closed
	let closed = Arc::new(AtomicUsize::new(0));
	let closed_clone = Arc::clone(&closed);
	peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
		if s == RTCPeerConnectionState::Failed || s == RTCPeerConnectionState::Closed {
			closed_clone.store(1, Ordering::SeqCst);
		}
		Box::pin(async {})
	}));

	while closed.load(Ordering::SeqCst) == 0 {
		tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
	}

	Ok(())
}

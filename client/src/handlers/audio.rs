use crate::{
    handlers::func::{Function, HandlerResult},
    router::SocketWriter,
};

pub struct Audio;
impl Function for Audio {
    // #[must_use]
    // #[allow(
    // elided_named_lifetimes,
    // clippy::type_complexity,
    // clippy::type_repetition_in_bounds
    // )]
    fn handler<'a, 'async_trait>(
        args: serde_json::Value,
        socket: &'a mut SocketWriter,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = HandlerResult>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'a: 'async_trait,
    {
        todo!()
    }
}

pub(crate) async fn handler(
    args: serde_json::Value,
    socket: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        tokio_tungstenite::tungstenite::Message,
    >,
) -> Result<(), anyhow::Error> {
    todo!()
}

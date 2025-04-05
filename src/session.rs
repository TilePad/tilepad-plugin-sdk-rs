use std::task::{Poll, ready};

use futures::Stream;
use serde::Serialize;
use thiserror::Error;

use crate::{
    protocol::{ClientPluginMessage, InspectorContext, PluginId, ServerPluginMessage},
    ws::{WsMessage, WsRx, WsTx},
};

#[derive(Debug, Error)]
pub enum SessionError {
    /// Error while serializing a message
    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    /// Channel to send messages has closed, the server and socket
    /// are no longer reachable
    #[error("session closed")]
    Closed,

    /// Got an unexpected message from the server
    #[error("unexpected message")]
    UnexpectedMessage,
}

/// Handle to send messages on behalf of the plugin
#[derive(Clone)]
pub struct PluginSessionHandle {
    tx: WsTx,
}

impl PluginSessionHandle {
    pub(crate) fn new(tx: WsTx) -> Self {
        Self { tx }
    }
}

impl PluginSessionHandle {
    pub(crate) fn send_message(&self, msg: ClientPluginMessage) -> Result<(), SessionError> {
        let msg = serde_json::to_string(&msg)?;
        let message = WsMessage::text(msg);
        tracing::debug!(?message, "sending message to server");
        self.tx.send(message).map_err(|_| SessionError::Closed)?;
        Ok(())
    }

    pub(crate) fn register(&self, plugin_id: PluginId) -> Result<(), SessionError> {
        self.send_message(ClientPluginMessage::RegisterPlugin { plugin_id })?;
        Ok(())
    }

    pub fn get_properties(&self) -> Result<(), SessionError> {
        self.send_message(ClientPluginMessage::GetProperties {})?;
        Ok(())
    }

    pub fn set_properties<T>(&self, msg: T) -> Result<(), SessionError>
    where
        T: Serialize,
    {
        let properties = serde_json::to_value(msg)?;
        self.send_message(ClientPluginMessage::SetProperties { properties })
    }

    pub fn send_to_inspector<T>(&self, ctx: InspectorContext, msg: T) -> Result<(), SessionError>
    where
        T: Serialize,
    {
        let message = serde_json::to_value(msg)?;
        self.send_message(ClientPluginMessage::SendToInspector { ctx, message })
    }

    pub fn open_url(&self, url: String) -> Result<(), SessionError> {
        self.send_message(ClientPluginMessage::OpenUrl { url })
    }
}

pub(crate) struct PluginSessionRx {
    rx: WsRx,
}

impl PluginSessionRx {
    pub(crate) fn new(rx: WsRx) -> Self {
        Self { rx }
    }
}

impl Stream for PluginSessionRx {
    type Item = Result<ServerPluginMessage, SessionError>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            // Receive a websocket message
            let msg = match ready!(this.rx.poll_recv(cx)) {
                Some(value) => value,
                None => return Poll::Ready(None),
            };

            let msg = match msg {
                WsMessage::Text(utf8_bytes) => utf8_bytes,

                // Ping and pong are handled internally
                WsMessage::Ping(_) | WsMessage::Pong(_) | WsMessage::Frame(_) => continue,

                // Expecting a text based protocol
                WsMessage::Binary(_) => {
                    return Poll::Ready(Some(Err(SessionError::UnexpectedMessage)));
                }

                // Socket is closed
                WsMessage::Close(_) => return Poll::Ready(None),
            };

            tracing::debug!(?msg, "received message from server");

            let msg: ServerPluginMessage = match serde_json::from_str(msg.as_str()) {
                Ok(value) => value,
                Err(cause) => {
                    tracing::error!(?cause, "invalid or unknown message");
                    continue;
                }
            };

            return Poll::Ready(Some(Ok(msg)));
        }
    }
}

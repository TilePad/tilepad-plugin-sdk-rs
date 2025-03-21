use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, ready},
};

use crate::{
    TilepadPlugin,
    protocol::PluginMessageContext,
    ws::{WebSocket, WebSocketFuture, WsMessage, WsRx, WsTx},
};
use anyhow::anyhow;
use serde::Serialize;
use tokio::spawn;
use tracing::error;

use super::protocol::{ClientPluginMessage, ServerPluginMessage};

pub type PluginSessionRef = Arc<PluginSession>;

pub struct PluginSession {
    /// Access to the plugins registry
    plugin: Arc<TilepadPlugin>,
    /// Sender to send messages to the session socket
    tx: WsTx,
}

impl PluginSession {
    pub(crate) async fn run(plugin: Arc<TilepadPlugin>, socket: WebSocket) -> anyhow::Result<()> {
        // Create and spawn a future for the websocket
        let (ws_future, ws_rx, ws_tx) = WebSocketFuture::new(socket);
        spawn(async move {
            if let Err(cause) = ws_future.await {
                error!(?cause, "error running device websocket future");
            }
        });

        let session = Arc::new(PluginSession { plugin, tx: ws_tx });

        session.send_message(ClientPluginMessage::RegisterPlugin {
            plugin_id: session.plugin.plugin_id.clone(),
        })?;

        // Create and spawn a future to process session messages
        let session_future = PluginSessionFuture { session, rx: ws_rx };

        session_future.await?;
        Ok(())
    }

    pub(crate) fn send_message(&self, msg: ClientPluginMessage) -> anyhow::Result<()> {
        let msg = serde_json::to_string(&msg)?;
        let message = WsMessage::text(msg);
        self.tx.send(message)?;
        Ok(())
    }

    pub fn get_properties(&self) -> anyhow::Result<()> {
        self.send_message(ClientPluginMessage::GetProperties {})?;
        Ok(())
    }

    pub fn set_properties<T>(&self, msg: T) -> anyhow::Result<()>
    where
        T: Serialize,
    {
        let properties = serde_json::to_value(msg)?;
        self.send_message(ClientPluginMessage::SetProperties { properties })
    }

    pub fn send_to_inspector<T>(&self, ctx: PluginMessageContext, msg: T) -> anyhow::Result<()>
    where
        T: Serialize,
    {
        let message = serde_json::to_value(msg)?;
        self.send_message(ClientPluginMessage::SendToInspector { ctx, message })
    }

    pub fn handle_message(self: &Arc<Self>, message: ServerPluginMessage) {
        match message {
            ServerPluginMessage::Properties { properties } => {
                if let Some(on_properties) = &self.plugin.on_properties {
                    tokio::spawn(on_properties.on_properties(
                        self.plugin.clone(),
                        self.clone(),
                        properties,
                    ));
                }
            }
            ServerPluginMessage::TileClicked { ctx, properties } => {
                if let Some(on_tile_clicked) = &self.plugin.on_tile_clicked {
                    tokio::spawn(on_tile_clicked.on_tile_clicked(
                        self.plugin.clone(),
                        self.clone(),
                        ctx,
                        properties,
                    ));
                }
            }
            ServerPluginMessage::RecvFromInspector { ctx, message } => {
                if let Some(on_inspector_message) = &self.plugin.on_inspector_message {
                    tokio::spawn(on_inspector_message.on_inspector_message(
                        self.plugin.clone(),
                        self.clone(),
                        ctx,
                        message,
                    ));
                }
            }
            ServerPluginMessage::Registered { plugin_id: _ } => {
                // Request the current properties
                _ = self.get_properties();

                if let Some(on_init) = &self.plugin.on_init {
                    tokio::spawn(on_init.on_init(self.plugin.clone(), self.clone()));
                }
            }
        }
    }
}

/// Futures that processes messages for a device session
pub struct PluginSessionFuture {
    session: PluginSessionRef,
    rx: WsRx,
}

impl Future for PluginSessionFuture {
    type Output = anyhow::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        while let Some(msg) = ready!(this.rx.poll_recv(cx)) {
            let message = match msg {
                WsMessage::Text(utf8_bytes) => utf8_bytes,

                // Ping and pong are handled internally
                WsMessage::Ping(_) | WsMessage::Pong(_) | WsMessage::Frame(_) => continue,

                // Expecting a text based protocol
                WsMessage::Binary(_) => {
                    return Poll::Ready(Err(anyhow!("unexpected binary message")));
                }

                // Socket is closed
                WsMessage::Close(_) => return Poll::Ready(Ok(())),
            };

            let msg: ServerPluginMessage = serde_json::from_str(message.as_str())?;
            this.session.handle_message(msg);
        }

        // No more messages, session is terminated
        Poll::Ready(Ok(()))
    }
}

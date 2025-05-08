use std::task::{Poll, ready};

use futures_util::Stream;
use serde::Serialize;
use thiserror::Error;
use tokio::sync::oneshot;

use crate::{
    protocol::{
        ClientPluginMessage, InspectorContext, PluginId, ServerPluginMessage, TileIcon, TileId,
        TileLabel,
    },
    subscription::{Subscriber, Subscriptions},
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
    subscriptions: Subscriptions,
}

impl PluginSessionHandle {
    pub(crate) fn new(tx: WsTx, subscriptions: Subscriptions) -> Self {
        Self { tx, subscriptions }
    }
}

impl PluginSessionHandle {
    /// Sends a message over the plugin websocket
    pub(crate) fn send_message(&self, msg: ClientPluginMessage) -> Result<(), SessionError> {
        let msg = serde_json::to_string(&msg)?;
        let message = WsMessage::text(msg);
        tracing::debug!(?message, "sending message to server");
        self.tx.send(message).map_err(|_| SessionError::Closed)?;
        Ok(())
    }

    /// Registers the plugin with the plugin server
    pub(crate) fn register(&self, plugin_id: PluginId) -> Result<(), SessionError> {
        self.send_message(ClientPluginMessage::RegisterPlugin { plugin_id })?;
        Ok(())
    }

    /// Requests the current plugin properties from the server
    pub fn request_properties(&self) -> Result<(), SessionError> {
        self.send_message(ClientPluginMessage::GetProperties {})?;
        Ok(())
    }

    /// Requests the current properties from tilepad waiting until
    /// the response is retrieved and returns that
    pub async fn get_properties(&self) -> Result<serde_json::Value, SessionError> {
        let (tx, rx) = oneshot::channel();

        self.subscriptions.add(Subscriber::new(
            |msg| matches!(msg, ServerPluginMessage::Properties { .. }),
            tx,
        ));

        self.request_properties()?;

        // Wait for the response message
        let msg = rx.await.map_err(|_| SessionError::Closed)?;
        let msg = match msg {
            ServerPluginMessage::Properties { properties } => properties,
            _ => return Err(SessionError::UnexpectedMessage),
        };

        Ok(msg)
    }

    /// Sets the properties for the plugin
    ///
    /// This replaces the stored properties object with the
    /// provided `properties`.
    ///
    /// Use [PluginSessionHandle::set_properties_partial] to perform a partial update
    pub fn set_properties<T>(&self, properties: T) -> Result<(), SessionError>
    where
        T: Serialize,
    {
        let properties = serde_json::to_value(properties)?;
        self.send_message(ClientPluginMessage::SetProperties {
            properties,
            partial: false,
        })
    }

    /// Sets the properties for the plugin
    ///
    /// This performs a partial update, merging the existing
    /// plugin properties with the specified `properties`
    ///
    /// Use [PluginSessionHandle::set_properties] to replace the properties completely
    pub fn set_properties_partial<T>(&self, properties: T) -> Result<(), SessionError>
    where
        T: Serialize,
    {
        let properties = serde_json::to_value(properties)?;
        self.send_message(ClientPluginMessage::SetProperties {
            properties,
            partial: true,
        })
    }

    /// Requests the specified tile properties from the server
    pub fn request_tile_properties(&self, tile_id: TileId) -> Result<(), SessionError> {
        self.send_message(ClientPluginMessage::GetTileProperties { tile_id })?;
        Ok(())
    }

    /// Requests the current properties for a tile from tilepad waiting until
    /// the response is retrieved and returns that
    pub async fn get_tile_properties(
        &self,
        tile_id: TileId,
    ) -> Result<serde_json::Value, SessionError> {
        let (tx, rx) = oneshot::channel();

        self.subscriptions.add(Subscriber::new(
            move |msg| match msg {
                ServerPluginMessage::TileProperties {
                    tile_id: other_id, ..
                } => other_id.eq(&tile_id),
                _ => false,
            },
            tx,
        ));

        self.request_tile_properties(tile_id)?;

        // Wait for the response message
        let msg = rx.await.map_err(|_| SessionError::Closed)?;
        let msg = match msg {
            ServerPluginMessage::TileProperties { properties, .. } => properties,
            _ => return Err(SessionError::UnexpectedMessage),
        };

        Ok(msg)
    }

    /// Sets the properties for the specified tile
    ///
    /// You can only update tiles that are using an action
    /// from your plugin
    ///
    /// This replaces the stored properties object with the
    /// provided `properties`.
    ///
    /// Use [PluginSessionHandle::set_tile_properties_partial] to perform a partial update
    pub fn set_tile_properties<T>(&self, tile_id: TileId, properties: T) -> Result<(), SessionError>
    where
        T: Serialize,
    {
        let properties = serde_json::to_value(properties)?;
        self.send_message(ClientPluginMessage::SetTileProperties {
            tile_id,
            properties,
            partial: false,
        })
    }

    /// Sets the properties for the specified tile
    ///
    /// You can only update tiles that are using an action
    /// from your plugin
    ///
    /// This performs a partial update, merging the existing
    /// plugin properties with the specified `properties`
    ///
    /// Use [PluginSessionHandle::set_tile_properties] to replace the properties completely
    pub fn set_tile_properties_partial<T>(
        &self,
        tile_id: TileId,
        properties: T,
    ) -> Result<(), SessionError>
    where
        T: Serialize,
    {
        let properties = serde_json::to_value(properties)?;
        self.send_message(ClientPluginMessage::SetTileProperties {
            tile_id,
            properties,
            partial: true,
        })
    }

    /// Sets the icon for a specific tile
    ///
    /// You can only update tiles that are using an action
    /// from your plugin
    pub fn set_tile_icon(&self, tile_id: TileId, icon: TileIcon) -> Result<(), SessionError> {
        self.send_message(ClientPluginMessage::SetTileIcon { tile_id, icon })
    }

    /// Sets the label for a specific tile
    ///
    /// You can only update tiles that are using an action
    /// from your plugin
    pub fn set_tile_label(&self, tile_id: TileId, label: TileLabel) -> Result<(), SessionError> {
        self.send_message(ClientPluginMessage::SetTileLabel { tile_id, label })
    }

    /// Sends a message to the plugin inspector UI at the provided
    /// inspector context
    pub fn send_to_inspector<T>(&self, ctx: InspectorContext, msg: T) -> Result<(), SessionError>
    where
        T: Serialize,
    {
        let message = serde_json::to_value(msg)?;
        self.send_message(ClientPluginMessage::SendToInspector { ctx, message })
    }

    /// Tells tilepad to open the provided `url` in the
    /// default browser
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

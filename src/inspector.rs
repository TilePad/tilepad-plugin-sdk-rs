use serde::Serialize;

use crate::{
    protocol::{ClientPluginMessage, InspectorContext},
    session::{PluginSessionHandle, SessionError},
};

/// Reference to an inspector window that can be
/// used to send messages
#[derive(Clone)]
pub struct Inspector {
    /// Plugin session handle the inspector is connected through
    pub session: PluginSessionHandle,
    /// Context data for the inspector
    pub ctx: InspectorContext,
}

impl Inspector {
    /// Send a JSON serializable message `msg` to the inspector window
    pub fn send<M>(&self, msg: M) -> Result<(), SessionError>
    where
        M: Serialize,
    {
        let message = serde_json::to_value(msg)?;
        self.session
            .send_message(ClientPluginMessage::SendToInspector {
                ctx: self.ctx.clone(),
                message,
            })
    }
}

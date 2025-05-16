use serde::Serialize;

use crate::{
    protocol::{ClientPluginMessage, DisplayContext},
    session::{PluginSessionHandle, SessionError},
};

/// Reference to an display that can be
/// used to send messages
#[derive(Clone)]
pub struct Display {
    /// Plugin session handle the display is connected through
    pub session: PluginSessionHandle,
    /// Context data for the display
    pub ctx: DisplayContext,
}

impl Display {
    /// Send a JSON serializable message `msg` to the display
    pub fn send<M>(&self, msg: M) -> Result<(), SessionError>
    where
        M: Serialize,
    {
        let message = serde_json::to_value(msg)?;
        self.session
            .send_message(ClientPluginMessage::SendToDisplay {
                ctx: self.ctx.clone(),
                message,
            })
    }
}

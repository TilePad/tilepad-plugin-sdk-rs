use serde::Serialize;

use crate::{
    protocol::{ClientPluginMessage, InspectorContext},
    session::{PluginSessionHandle, SessionError},
};

#[derive(Clone)]
pub struct Inspector {
    pub session: PluginSessionHandle,
    pub ctx: InspectorContext,
}

impl Inspector {
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

use crate::{
    inspector::Inspector,
    protocol::{DeepLinkContext, TileInteractionContext},
    session::PluginSessionHandle,
};

#[allow(unused_variables)]
pub trait Plugin {
    /// Invoked when the plugin is successfully registered with the
    /// Tilepad application and has a usable session
    ///
    /// # Arguments
    /// * `session` - The current session
    fn on_registered(&self, session: &PluginSessionHandle) {}

    /// Invoked when the plugin properties are received from Tilepad,
    /// this will occur when the plugin calls `session.get_properties`
    /// but also once when the plugin is first registered
    ///
    /// # Arguments
    /// * `session` - The current session
    /// * `properties` - The current plugin properties
    fn on_properties(&self, session: &PluginSessionHandle, properties: serde_json::Value) {}

    /// Invoked when the plugin receives a message from the inspector,
    /// this message structure is defined by the developer   
    ///
    /// # Arguments
    /// * `session` - The current session
    /// * `ctx`     - Contextual information about the inspector (Which tile is selected, which folder, which profile etc)
    /// * `message` - The message sent from the inspector
    fn on_inspector_message(
        &self,
        session: &PluginSessionHandle,
        inspector: Inspector,
        message: serde_json::Value,
    ) {
    }

    /// Invoked when the inspector is opened for a tile
    ///
    /// # Arguments
    /// * `session` - The current session
    /// * `ctx`     - Contextual information about the inspector (Which tile is selected, which folder, which profile etc)
    fn on_inspector_open(&self, session: &PluginSessionHandle, inspector: Inspector) {}

    /// Invoked when the inspector is closed for a tile
    ///
    /// # Arguments
    /// * `session` - The current session
    /// * `ctx`     - Contextual information about the inspector (Which tile is selected, which folder, which profile etc)
    fn on_inspector_close(&self, session: &PluginSessionHandle, inspector: Inspector) {}

    /// Invoked when a deep link is received for the plugin
    ///
    /// # Arguments
    /// * `session` - The current session
    /// * `ctx`     - Information about the deep-link
    fn on_deep_link(&self, session: &PluginSessionHandle, ctx: DeepLinkContext) {}

    /// Invoked when a tile is clicked on a device
    ///
    /// # Arguments
    /// * `session`    - The current session
    /// * `ctx`        - Contextual information about tile clicked tile (Device, action, etc)
    /// * `properties` - The current tile properties at the time of clicking
    fn on_tile_clicked(
        &self,
        session: &PluginSessionHandle,
        ctx: TileInteractionContext,
        properties: serde_json::Value,
    ) {
    }
}

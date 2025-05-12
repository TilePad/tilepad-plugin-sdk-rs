use crate::{
    inspector::Inspector,
    protocol::{DeepLinkContext, DeviceId, TileId, TileInteractionContext, TileModel},
    session::PluginSessionHandle,
};

/// Trait implemented by your plugin
#[allow(unused_variables)]
pub trait Plugin {
    /// Invoked when the plugin is successfully registered with the
    /// Tilepad application and has a usable session
    ///
    /// # Arguments
    /// * `session` - The current session
    fn on_registered(&mut self, session: &PluginSessionHandle) {}

    /// Invoked when the plugin properties are received from Tilepad,
    /// this will occur when the plugin calls `session.request_properties` or `session.get_properties`
    /// but also once when the plugin is first registered
    ///
    /// # Arguments
    /// * `session` - The current session
    /// * `properties` - The current plugin properties
    fn on_properties(&mut self, session: &PluginSessionHandle, properties: serde_json::Value) {}

    /// Invoked when a tiles properties are received from Tilepad,
    /// this will occur when the plugin calls [PluginSessionHandle::request_tile_properties] or  [PluginSessionHandle::get_tile_properties]
    ///
    /// # Arguments
    /// * `session` - The current session
    /// * `tile_id` - ID of the tile that the properties are for
    /// * `properties` - The current plugin properties
    fn on_tile_properties(
        &mut self,
        session: &PluginSessionHandle,
        tile_id: TileId,
        properties: serde_json::Value,
    ) {
    }

    /// Invoked when the plugin receives a message from the inspector,
    /// this message structure is defined by the developer   
    ///
    /// # Arguments
    /// * `session` - The current session
    /// * `ctx`     - Contextual information about the inspector (Which tile is selected, which folder, which profile etc)
    /// * `message` - The message sent from the inspector
    fn on_inspector_message(
        &mut self,
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
    fn on_inspector_open(&mut self, session: &PluginSessionHandle, inspector: Inspector) {}

    /// Invoked when the inspector is closed for a tile
    ///
    /// # Arguments
    /// * `session` - The current session
    /// * `ctx`     - Contextual information about the inspector (Which tile is selected, which folder, which profile etc)
    fn on_inspector_close(&mut self, session: &PluginSessionHandle, inspector: Inspector) {}

    /// Invoked when a deep link is received for the plugin
    ///
    /// # Arguments
    /// * `session` - The current session
    /// * `ctx`     - Information about the deep-link
    fn on_deep_link(&mut self, session: &PluginSessionHandle, ctx: DeepLinkContext) {}

    /// Invoked when a tile is clicked on a device
    ///
    /// # Arguments
    /// * `session`    - The current session
    /// * `ctx`        - Contextual information about tile clicked tile (Device, action, etc)
    /// * `properties` - The current tile properties at the time of clicking
    fn on_tile_clicked(
        &mut self,
        session: &PluginSessionHandle,
        ctx: TileInteractionContext,
        properties: serde_json::Value,
    ) {
    }

    /// Invoked when the visible tiles on a device change
    ///
    /// # Arguments
    /// * `session`   - The current session
    /// * `device_id` - ID of the device the tiles are for
    /// * `tiles`     - The current tiles of the device
    fn on_device_tiles(
        &mut self,
        session: &PluginSessionHandle,
        device_id: DeviceId,
        tiles: Vec<TileModel>,
    ) {
    }
    /// Invoked when the list of visible tiles is received
    /// this will occur when the plugin calls [PluginSessionHandle::request_tile_properties] or  [PluginSessionHandle::get_tile_properties]
    ///
    /// # Arguments
    /// * `session`   - The current session
    /// * `tiles`     - The current tiles of the device
    fn on_visible_tiles(&mut self, session: &PluginSessionHandle, tiles: Vec<TileModel>) {}
}

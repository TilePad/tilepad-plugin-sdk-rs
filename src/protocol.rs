use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type PluginId = String;
pub type ActionId = String;

pub type ProfileId = Uuid;
pub type FolderId = Uuid;
pub type DeviceId = Uuid;
pub type TileId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMessageContext {
    pub profile_id: ProfileId,
    pub folder_id: FolderId,

    pub plugin_id: PluginId,
    pub action_id: ActionId,

    pub tile_id: TileId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceMessageContext {
    pub device_id: DeviceId,

    pub plugin_id: PluginId,
    pub action_id: ActionId,

    pub tile_id: TileId,
}

/// Plugin message coming from the client side
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum ClientPluginMessage {
    /// Register the current plugin with the server
    RegisterPlugin { plugin_id: PluginId },

    /// Request the current plugin properties
    GetProperties,

    /// Set the properties for the plugin (Partial update)
    SetProperties { properties: serde_json::Value },

    /// Send data to the current inspector window
    SendToInspector {
        /// Inspector context
        ctx: PluginMessageContext,
        /// Message to send the inspector
        message: serde_json::Value,
    },
}

/// Plugin message coming from the server side
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ServerPluginMessage {
    /// Plugin has registered with the server
    Registered { plugin_id: PluginId },

    /// Properties received from the server
    Properties { properties: serde_json::Value },

    /// Tile was clicked on a remote device
    TileClicked {
        ctx: DeviceMessageContext,
        properties: serde_json::Value,
    },

    /// Got a message from the inspector
    RecvFromInspector {
        ctx: PluginMessageContext,
        message: serde_json::Value,
    },
}

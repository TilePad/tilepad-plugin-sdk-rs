use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type PluginId = String;
pub type ActionId = String;

pub type ProfileId = Uuid;
pub type FolderId = Uuid;
pub type DeviceId = Uuid;
pub type TileId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectorContext {
    pub profile_id: ProfileId,
    pub folder_id: FolderId,

    pub plugin_id: PluginId,
    pub action_id: ActionId,

    pub tile_id: TileId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileInteractionContext {
    pub device_id: DeviceId,

    pub plugin_id: PluginId,
    pub action_id: ActionId,

    pub tile_id: TileId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLinkContext {
    pub url: String,
    pub host: Option<String>,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

/// Plugin message coming from the client side
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub(crate) enum ClientPluginMessage {
    /// Register the current plugin with the server
    RegisterPlugin { plugin_id: PluginId },

    /// Request the current plugin properties
    GetProperties,

    /// Set the properties for the plugin (Partial update)
    SetProperties { properties: serde_json::Value },

    /// Send data to the current inspector window
    SendToInspector {
        /// Inspector context
        ctx: InspectorContext,
        /// Message to send the inspector
        message: serde_json::Value,
    },

    /// Open a URL
    OpenUrl { url: String },
}

/// Plugin message coming from the server side
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub(crate) enum ServerPluginMessage {
    /// Plugin has registered with the server
    Registered {
        #[allow(unused)]
        plugin_id: PluginId,
    },

    /// Properties received from the server
    Properties { properties: serde_json::Value },

    /// Tile was clicked on a remote device
    TileClicked {
        ctx: TileInteractionContext,
        properties: serde_json::Value,
    },

    /// Got a message from the inspector
    RecvFromInspector {
        ctx: InspectorContext,
        message: serde_json::Value,
    },

    /// Inspector was opened
    InspectorOpen { ctx: InspectorContext },

    /// Inspector was closed
    InspectorClose { ctx: InspectorContext },

    /// Received a deep link message for the plugin
    DeepLink { ctx: DeepLinkContext },
}

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

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum TileIcon {
    /// No icon
    #[default]
    None,

    /// Icon from a specific plugin path
    PluginIcon {
        /// ID of the plugin the icon is from
        plugin_id: PluginId,
        /// Path to the icon file
        icon: String,
    },

    /// Use an icon from an icon pack
    IconPack {
        /// ID of the icon pack
        pack_id: String,
        /// Path to the icon file
        path: String,
    },

    // Image at some remote URL
    Url {
        src: String,
    },

    /// User uploaded file
    Uploaded {
        /// Path to the uploaded file
        path: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TileLabel {
    pub enabled: bool,
    pub label: String,
    pub align: LabelAlign,

    pub font: String,
    pub font_size: u32,

    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub outline: bool,

    pub color: String,
    pub outline_color: String,
}

impl Default for TileLabel {
    fn default() -> Self {
        Self {
            enabled: true,
            label: Default::default(),
            align: Default::default(),
            font: "Roboto".to_string(),
            font_size: 10,
            bold: false,
            italic: false,
            underline: false,
            outline: true,
            color: "#ffffff".to_string(),
            outline_color: "#000000".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LabelAlign {
    #[default]
    Bottom,
    Middle,
    Top,
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
    SetProperties {
        properties: serde_json::Value,

        /// Whether to treat the properties update as a partial update
        partial: bool,
    },

    /// Send data to the current inspector window
    SendToInspector {
        /// Inspector context
        ctx: InspectorContext,
        /// Message to send the inspector
        message: serde_json::Value,
    },

    /// Open a URL
    OpenUrl { url: String },

    /// Request the current properties for a tile
    GetTileProperties {
        /// ID of the tile to get properties for
        tile_id: TileId,
    },

    /// Set the current properties for a tile
    SetTileProperties {
        /// ID of the tile to set properties for
        tile_id: TileId,
        /// Properties for the tile
        properties: serde_json::Value,
        /// Whether to treat the properties update as a partial update
        partial: bool,
    },

    /// Set the current icon for a tile
    SetTileIcon { tile_id: TileId, icon: TileIcon },

    /// Set the current label for a tile
    SetTileLabel { tile_id: TileId, label: TileLabel },
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

    /// Properties requested for a tile
    TileProperties {
        tile_id: TileId,
        properties: serde_json::Value,
    },
}

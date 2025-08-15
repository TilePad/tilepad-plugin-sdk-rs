use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type PluginId = String;
pub type ActionId = String;

pub type ProfileId = Uuid;
pub type FolderId = Uuid;
pub type DeviceId = Uuid;
pub type TileId = Uuid;
pub type JsonObject = serde_json::Map<String, serde_json::Value>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InspectorContext {
    pub profile_id: ProfileId,
    pub folder_id: FolderId,

    pub plugin_id: PluginId,
    pub action_id: ActionId,

    pub tile_id: TileId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisplayContext {
    pub device_id: DeviceId,
    pub plugin_id: PluginId,
    pub action_id: ActionId,
    pub tile_id: TileId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileModel {
    /// Unique ID of the tile
    pub id: TileId,

    /// Configuration for the tile and how it appears in the UI
    pub config: TileConfig,

    /// Properties / settings defined on this specific tile
    pub properties: JsonObject,

    /// ID of the folder this tile is within
    pub folder_id: FolderId,

    /// ID of the plugin the `action_id` is apart of
    pub plugin_id: PluginId,
    /// ID of the action within the plugin to execute
    pub action_id: ActionId,

    /// Position of the tile
    pub position: TilePosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileConfig {
    /// Icon to use
    pub icon: TileIcon,
    /// Label to display on top of the tile
    pub label: TileLabel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileIconOptions {
    pub padding: u32,
    pub background_color: String,
    pub border_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TilePosition {
    /// Row within the UI to display at
    pub row: u32,
    /// Column within the UI to display at
    pub column: u32,
    /// Number of rows to span
    pub row_span: u32,
    /// Number of columns to span
    pub column_span: u32,
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
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TileLabel {
    pub enabled: Option<bool>,
    pub label: Option<String>,
    pub align: Option<LabelAlign>,

    pub font: Option<String>,
    pub font_size: Option<u32>,

    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub outline: Option<bool>,

    pub color: Option<String>,
    pub outline_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
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

    /// Send data to a specific display
    SendToDisplay {
        /// Inspector context
        ctx: DisplayContext,
        /// Message to send the display
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

    /// Get all currently visible tiles
    GetVisibleTiles,

    /// Display an icon on connected devices
    DisplayIndicator {
        /// ID of the device to display on
        device_id: Uuid,
        /// ID of the tile to display it on
        tile_id: Uuid,
        /// Indicator to display
        indicator: DeviceIndicator,
        /// Duration in milliseconds to display the
        /// indicator for
        duration: u32,
    },
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

    /// Got a message from a display
    RecvFromDisplay {
        ctx: DisplayContext,
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

    /// Selection of tiles for a device has changed
    DeviceTiles {
        /// ID of the device that changes
        device_id: DeviceId,
        /// Tiles that are now visible on the device
        tiles: Vec<TileModel>,
    },

    VisibleTiles {
        /// Tiles that are currently visible
        tiles: Vec<TileModel>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DeviceIndicator {
    Error,
    Success,
    Warning,
    Loading,
    /// Clear the active indicator
    None,
}

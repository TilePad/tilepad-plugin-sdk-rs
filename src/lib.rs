//! # Tilepad SDK
//!
//! ```no_run
//! use tilepad_plugin_sdk::{Plugin, PluginSessionHandle, start_plugin, setup_tracing};
//! use tokio::task::LocalSet;
//!
//! #[derive(Default)]
//! struct MyPlugin {}
//!
//! impl Plugin for MyPlugin {
//!     // TODO: Implement your desired methods
//! }
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() {
//!     setup_tracing();
//!
//!     let local_set = LocalSet::new();
//!     let plugin = MyPlugin::default();
//!
//!     local_set.run_until(start_plugin(plugin)).await;
//! }
//! ```

use clap::Parser;
use futures_util::StreamExt;
use protocol::ServerPluginMessage;
use session::PluginSessionRx;
use subscription::Subscriptions;
use tokio::join;
use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest};

use tracing_subscriber::EnvFilter;
use ws::WebSocketFuture;

// Provide tracing modules to the implementor
pub use tracing;
pub use tracing_subscriber;

// Module re-exports
pub use display::Display;
pub use inspector::Inspector;
pub use plugin::Plugin;
pub use protocol::*;
pub use session::{PluginSessionHandle, SessionError};

mod display;
mod inspector;
mod plugin;
mod protocol;
mod session;
mod subscription;
mod ws;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// ID of the plugin to connect as
    #[arg(long)]
    plugin_id: String,

    /// Plugin server connection host URL
    #[arg(long)]
    connect_url: String,
}

pub async fn start_plugin<P>(plugin: P)
where
    P: Plugin,
{
    // Accept the command line arguments
    let args = Args::parse();

    // Connect to the server socket
    let client_request = args
        .connect_url
        .into_client_request()
        .expect("failed to create client request");
    let (socket, _response) = connect_async(client_request)
        .await
        .expect("failed to connect to plugin server");

    // Create and spawn a future for the websocket
    let (ws_future, ws_rx, ws_tx) = WebSocketFuture::new(socket);

    // Create message subscriptions store
    let subscriptions = Subscriptions::default();

    // Wrap the websocket handle with the custom protocol
    let handle = PluginSessionHandle::new(ws_tx, subscriptions.clone());

    // Send registration message
    handle
        .register(args.plugin_id)
        .expect("failed to register plugin");

    let msg_rx = PluginSessionRx::new(ws_rx);

    let socket_future = run_websocket(ws_future);
    let handle_future = run_handler(plugin, handle, subscriptions, msg_rx);

    join!(socket_future, handle_future);
}

/// Helper to run the websocket and emit a log in the case of error
async fn run_websocket(ws_future: WebSocketFuture) {
    if let Err(cause) = ws_future.await {
        tracing::error!(?cause, "error running device websocket future");
    }
}

/// Handle all incoming messages from the websocket
async fn run_handler<P>(
    mut plugin: P,
    handle: PluginSessionHandle,
    subscriptions: Subscriptions,
    mut msg_rx: PluginSessionRx,
) where
    P: Plugin,
{
    while let Some(msg) = msg_rx.next().await {
        let msg = match msg {
            Ok(value) => value,
            Err(cause) => {
                tracing::error!(?cause, "error processing server message");
                return;
            }
        };

        // Handle subscriptions
        subscriptions.apply(&msg);

        match msg {
            ServerPluginMessage::Registered { .. } => {
                handle
                    .request_properties()
                    .expect("failed to request initial properties");

                plugin.on_registered(&handle);
            }
            ServerPluginMessage::Properties { properties } => {
                plugin.on_properties(&handle, properties);
            }
            ServerPluginMessage::TileClicked { ctx, properties } => {
                plugin.on_tile_clicked(&handle, ctx, properties);
            }
            ServerPluginMessage::RecvFromInspector { ctx, message } => {
                plugin.on_inspector_message(
                    &handle,
                    Inspector {
                        ctx,
                        session: handle.clone(),
                    },
                    message,
                );
            }
            ServerPluginMessage::RecvFromDisplay { ctx, message } => {
                plugin.on_display_message(
                    &handle,
                    Display {
                        ctx,
                        session: handle.clone(),
                    },
                    message,
                );
            }
            ServerPluginMessage::InspectorOpen { ctx } => {
                plugin.on_inspector_open(
                    &handle,
                    Inspector {
                        ctx,
                        session: handle.clone(),
                    },
                );
            }
            ServerPluginMessage::InspectorClose { ctx } => {
                plugin.on_inspector_close(
                    &handle,
                    Inspector {
                        ctx,
                        session: handle.clone(),
                    },
                );
            }
            ServerPluginMessage::DeepLink { ctx } => {
                plugin.on_deep_link(&handle, ctx);
            }
            ServerPluginMessage::TileProperties {
                tile_id,
                properties,
            } => {
                plugin.on_tile_properties(&handle, tile_id, properties);
            }

            ServerPluginMessage::DeviceTiles { device_id, tiles } => {
                plugin.on_device_tiles(&handle, device_id, tiles);
            }

            ServerPluginMessage::VisibleTiles { tiles } => {
                plugin.on_visible_tiles(&handle, tiles);
            }
        }
    }

    subscriptions.clear();
}

pub fn setup_tracing() {
    let filter = EnvFilter::from_default_env();
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_env_filter(filter)
        .with_line_number(true)
        .with_thread_ids(false)
        .with_target(false)
        .with_ansi(false)
        .without_time()
        .finish();

    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber).expect("failed to setup tracing");
}

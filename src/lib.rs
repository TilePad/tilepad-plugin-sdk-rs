use clap::Parser;
use futures::StreamExt;
use inspector::Inspector;
use plugin::Plugin;
use protocol::ServerPluginMessage;
use session::{PluginSessionHandle, PluginSessionRx};
use subscription::Subscriptions;
use tokio::spawn;
use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest};

use ws::WebSocketFuture;

// Provide tracing modules to the implementor
pub use tracing;
pub use tracing_subscriber;

pub mod inspector;
pub mod plugin;
pub mod protocol;
pub mod session;
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
    spawn(async move {
        if let Err(cause) = ws_future.await {
            tracing::error!(?cause, "error running device websocket future");
        }
    });

    // Create message subscriptions store
    let subscriptions = Subscriptions::default();

    // Wrap the websocket handle with the custom protocol
    let handle = PluginSessionHandle::new(ws_tx, subscriptions.clone());

    // Send registration message
    handle
        .register(args.plugin_id)
        .expect("failed to register plugin");

    let mut msg_rx = PluginSessionRx::new(ws_rx);

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
            } => {}
        }
    }
}

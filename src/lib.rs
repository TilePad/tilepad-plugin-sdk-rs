use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use clap::Parser;
use futures::future::BoxFuture;
use protocol::{DeviceMessageContext, PluginMessageContext};
use socket::{PluginSession, PluginSessionRef};
use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest};
use ws::WebSocket;

pub mod protocol;
pub mod socket;
pub mod ws;

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

pub struct TilepadPluginBuilder {
    plugin_id: String,
    connect_url: String,
    extensions: AnyMap,
    on_inspector_message: Option<Box<dyn OnInspectorMessage>>,
    on_init: Option<Box<dyn OnInit>>,
    on_settings: Option<Box<dyn OnSettings>>,
    on_tile_clicked: Option<Box<dyn OnTileClicked>>,
}

type AnyMap = HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>;

impl TilepadPluginBuilder {
    pub fn new(plugin_id: String, connect_url: String) -> Self {
        Self {
            plugin_id,
            connect_url,
            extensions: Default::default(),
            on_init: None,
            on_inspector_message: None,
            on_settings: None,
            on_tile_clicked: None,
        }
    }

    pub fn build(self) -> TilepadPlugin {
        TilepadPlugin {
            plugin_id: self.plugin_id,
            connect_url: self.connect_url,
            extensions: self.extensions,
            on_init: self.on_init,
            on_inspector_message: self.on_inspector_message,
            on_settings: self.on_settings,
            on_tile_clicked: self.on_tile_clicked,
        }
    }

    pub fn add_extension<T: Send + Sync + 'static>(&mut self, val: T) {
        self.extensions.insert(TypeId::of::<T>(), Box::new(val));
    }

    pub fn extension<T: Send + Sync + 'static>(mut self, val: T) -> Self {
        self.add_extension(val);
        self
    }

    pub fn on_init<H>(mut self, handler: H) -> Self
    where
        H: OnInit + 'static,
    {
        self.on_init = Some(Box::new(handler));
        self
    }

    pub fn on_settings<H>(mut self, handler: H) -> Self
    where
        H: OnSettings + 'static,
    {
        self.on_settings = Some(Box::new(handler));
        self
    }

    pub fn on_tile_clicked<H>(mut self, handler: H) -> Self
    where
        H: OnTileClicked + 'static,
    {
        self.on_tile_clicked = Some(Box::new(handler));
        self
    }

    pub fn on_inspector_message<H>(mut self, handler: H) -> Self
    where
        H: OnInspectorMessage + 'static,
    {
        self.on_inspector_message = Some(Box::new(handler));
        self
    }
}

pub struct TilepadPlugin {
    plugin_id: String,
    connect_url: String,
    extensions: AnyMap,

    on_init: Option<Box<dyn OnInit>>,
    on_inspector_message: Option<Box<dyn OnInspectorMessage>>,
    on_settings: Option<Box<dyn OnSettings>>,
    on_tile_clicked: Option<Box<dyn OnTileClicked>>,
}

impl TilepadPlugin {
    pub fn builder() -> TilepadPluginBuilder {
        let args = Args::parse();

        TilepadPluginBuilder::new(args.plugin_id, args.connect_url)
    }

    pub fn extension<T>(&self) -> Option<T>
    where
        T: Clone + 'static,
    {
        self.extensions
            .get(&TypeId::of::<T>())
            .and_then(|boxed| (&**boxed as &(dyn Any + 'static)).downcast_ref())
            .cloned()
    }

    /// Run the tilepad plugin
    pub async fn run(self) {
        let plugin = Arc::new(self);

        let socket = connect_socket(plugin.connect_url.as_str())
            .await
            .expect("failed to connect to plugin server");
        PluginSession::run(plugin, socket)
            .await
            .expect("error while accessing plugin server");
    }
}

async fn connect_socket(connect_url: &str) -> anyhow::Result<WebSocket> {
    let (socket, _response) = connect_async(connect_url.into_client_request()?).await?;
    Ok(socket)
}

pub trait OnInit: Send + Sync + 'static {
    fn on_init(
        &self,
        plugin: Arc<TilepadPlugin>,
        session: PluginSessionRef,
    ) -> BoxFuture<'static, ()>;
}

pub trait OnSettings: Send + Sync + 'static {
    fn on_settings(
        &self,
        plugin: Arc<TilepadPlugin>,
        session: PluginSessionRef,
        settings: serde_json::Value,
    ) -> BoxFuture<'static, ()>;
}

pub trait OnInspectorMessage: Send + Sync + 'static {
    fn on_inspector_message(
        &self,
        plugin: Arc<TilepadPlugin>,
        session: PluginSessionRef,
        ctx: PluginMessageContext,
        message: serde_json::Value,
    ) -> BoxFuture<'static, ()>;
}

pub trait OnTileClicked: Send + Sync + 'static {
    fn on_tile_clicked(
        &self,
        plugin: Arc<TilepadPlugin>,
        session: PluginSessionRef,
        ctx: DeviceMessageContext,
        properties: serde_json::Value,
    ) -> BoxFuture<'static, ()>;
}

impl<F, Fut> OnInspectorMessage for F
where
    F: Fn(Arc<TilepadPlugin>, PluginSessionRef, PluginMessageContext, serde_json::Value) -> Fut
        + Send
        + Sync
        + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    fn on_inspector_message(
        &self,
        plugin: Arc<TilepadPlugin>,
        session: PluginSessionRef,
        ctx: PluginMessageContext,
        message: serde_json::Value,
    ) -> BoxFuture<'static, ()> {
        Box::pin(self(plugin, session, ctx, message))
    }
}

impl<F, Fut> OnSettings for F
where
    F: Fn(Arc<TilepadPlugin>, PluginSessionRef, serde_json::Value) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    fn on_settings(
        &self,
        plugin: Arc<TilepadPlugin>,
        session: PluginSessionRef,
        settings: serde_json::Value,
    ) -> BoxFuture<'static, ()> {
        Box::pin(self(plugin, session, settings))
    }
}

impl<F, Fut> OnInit for F
where
    F: Fn(Arc<TilepadPlugin>, PluginSessionRef) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    fn on_init(
        &self,
        plugin: Arc<TilepadPlugin>,
        session: PluginSessionRef,
    ) -> BoxFuture<'static, ()> {
        Box::pin(self(plugin, session))
    }
}

impl<F, Fut> OnTileClicked for F
where
    F: Fn(Arc<TilepadPlugin>, PluginSessionRef, DeviceMessageContext, serde_json::Value) -> Fut
        + Send
        + Sync
        + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    fn on_tile_clicked(
        &self,
        plugin: Arc<TilepadPlugin>,
        session: PluginSessionRef,
        ctx: DeviceMessageContext,
        properties: serde_json::Value,
    ) -> BoxFuture<'static, ()> {
        Box::pin(self(plugin, session, ctx, properties))
    }
}

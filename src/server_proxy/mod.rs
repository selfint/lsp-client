mod stdio_proxy;

pub use stdio_proxy::StdIOProxy;

use serde_json::Value;
use tokio::sync::{mpsc, oneshot};

pub type ToServerMsg = (Value, Option<oneshot::Sender<Value>>);
pub type ToServerChannel = mpsc::UnboundedSender<ToServerMsg>;

pub trait LspServerProxy {
    fn get_channel(&self) -> ToServerChannel;
}

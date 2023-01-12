use serde_json::Value;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

pub type ToServerChannel = mpsc::UnboundedSender<(Value, Option<oneshot::Sender<Value>>)>;

pub trait LspServerProxy {
    fn get_channel(&self) -> ToServerChannel;
}

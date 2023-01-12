use lsp_types::notification::Notification as LspNotification;
use lsp_types::request::Request as LspRequest;

use anyhow::Result;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::jsonrpc::types::Notification;
use crate::jsonrpc::types::Request;

pub type ToServerChannel = mpsc::UnboundedSender<(Value, Option<oneshot::Sender<Value>>)>;

pub trait LspServerProxy {
    fn start(&mut self) -> ToServerChannel;
}

async fn request<R: LspRequest>(
    params: R::Params,
    id: usize,
    to_server: ToServerChannel,
) -> Result<R::Result> {
    let request = serde_json::to_value(Request::new(R::METHOD, Some(params), Some(id)))?;

    let (sender, receiver) = oneshot::channel();

    to_server.send((request, Some(sender)))?;

    let response = receiver.await?;

    Ok(serde_json::from_value::<R::Result>(response)?)
}

fn notify<R: LspNotification>(params: R::Params, to_server: ToServerChannel) -> Result<()> {
    let request = serde_json::to_value(Notification::new(R::METHOD, Some(params)))?;

    to_server.send((request, None))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use lsp_types::{notification::Exit, request::Initialize, InitializeParams, InitializeResult};

    use super::*;

    struct MockLspServerProxy {
        hits: Arc<Mutex<HashMap<String, u32>>>,
    }

    impl MockLspServerProxy {
        fn new() -> Self {
            Self {
                hits: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    impl LspServerProxy for MockLspServerProxy {
        fn start(&mut self) -> ToServerChannel {
            let (sender, mut receiver) =
                mpsc::unbounded_channel::<(Value, Option<oneshot::Sender<Value>>)>();

            let hits = self.hits.clone();

            tokio::spawn(async move {
                while let Some((msg, response_channel)) = receiver.recv().await {
                    let method = msg
                        .get("method")
                        .expect("got msg without method")
                        .as_str()
                        .expect("got msg with non-str method");

                    *hits.lock().unwrap().entry(method.to_string()).or_insert(0) += 1;

                    match method {
                        "initialize" => {
                            response_channel
                                .expect("got initialize request with None response channel")
                                .send(serde_json::to_value(InitializeResult::default()).unwrap())
                                .expect("failed to send response to initialize request");
                        }
                        "exit" => {
                            dbg!("got exit");
                        }

                        _ => panic!("Got msg with unexpected method: '{}'", method),
                    }
                }
            });

            sender
        }
    }

    #[tokio::test]
    async fn test_request() {
        let mut proxy = MockLspServerProxy::new();
        let to_server = proxy.start();

        let response = request::<Initialize>(InitializeParams::default(), 1, to_server).await;

        assert!(response.is_ok());
        assert_eq!(InitializeResult::default(), response.unwrap());

        let hits = proxy.hits.lock().expect("failed to acquire proxy hits");

        assert!(hits.get("initialize").is_some());
        assert_eq!(hits.get("initialize").unwrap().to_owned(), 1);
    }

    #[tokio::test]
    async fn test_notify() {
        let mut proxy = MockLspServerProxy::new();
        let to_server = proxy.start();

        let response = notify::<Exit>((), to_server);

        assert!(response.is_ok());

        let mut timeout = 1000;
        loop {
            {
                if proxy
                    .hits
                    .lock()
                    .expect("failed to acquire proxy hits")
                    .get("exit")
                    .is_some()
                {
                    break;
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            timeout -= 10;
            if timeout <= 0 {
                break;
            }
        }

        let hits = proxy.hits.lock().expect("failed to acquire proxy hits");
        assert!(hits.get("exit").is_some());
        assert_eq!(hits.get("exit").unwrap().to_owned(), 1);
    }
}

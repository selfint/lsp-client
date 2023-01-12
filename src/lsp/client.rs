use lsp_types::notification::Notification as LspNotification;
use lsp_types::request::Request as LspRequest;

use anyhow::Result;
use tokio::sync::oneshot;

use crate::jsonrpc::types::Notification;
use crate::jsonrpc::types::Request;

use crate::lsp::server_proxy::ToServerChannel;

use super::server_proxy::LspServerProxy;

pub struct Client {
    to_server: ToServerChannel,
}

impl Client {
    pub fn new(proxy: &impl LspServerProxy) -> Self {
        Self {
            to_server: proxy.get_channel(),
        }
    }

    pub async fn request<R: LspRequest>(&self, params: R::Params, id: usize) -> Result<R::Result> {
        let request = serde_json::to_value(Request::new(R::METHOD, Some(params), Some(id)))?;

        let (sender, receiver) = oneshot::channel();

        self.to_server.send((request, Some(sender)))?;

        let response = receiver.await?;

        Ok(serde_json::from_value::<R::Result>(response)?)
    }

    pub fn notify<R: LspNotification>(&self, params: R::Params) -> Result<()> {
        let request = serde_json::to_value(Notification::new(R::METHOD, Some(params)))?;

        self.to_server.send((request, None))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use serde_json::Value;
    use tokio::sync::mpsc;
    use tokio::sync::oneshot;

    use lsp_types::{notification::Exit, request::Initialize, InitializeParams, InitializeResult};

    use super::*;
    use crate::lsp::server_proxy::LspServerProxy;

    struct MockLspServerProxy {
        hits: Arc<Mutex<HashMap<String, u32>>>,
        to_server: ToServerChannel,
    }

    impl MockLspServerProxy {
        fn new() -> Self {
            let (to_server, mut receiver) =
                mpsc::unbounded_channel::<(Value, Option<oneshot::Sender<Value>>)>();

            let hits = Arc::new(Mutex::new(HashMap::new()));
            let hits_2 = Arc::clone(&hits);

            tokio::spawn(async move {
                while let Some((msg, response_channel)) = receiver.recv().await {
                    let method = msg
                        .get("method")
                        .expect("got msg without method")
                        .as_str()
                        .expect("got msg with non-str method");

                    *hits_2
                        .lock()
                        .unwrap()
                        .entry(method.to_string())
                        .or_insert(0) += 1;

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

            Self { hits, to_server }
        }
    }

    impl LspServerProxy for MockLspServerProxy {
        fn get_channel(&self) -> ToServerChannel {
            self.to_server.clone()
        }
    }

    #[tokio::test]
    async fn test_request() {
        let proxy = MockLspServerProxy::new();

        let client = Client::new(&proxy);

        let response = client
            .request::<Initialize>(InitializeParams::default(), 1)
            .await;

        assert!(response.is_ok());
        assert_eq!(InitializeResult::default(), response.unwrap());

        let hits = proxy.hits.lock().expect("failed to acquire proxy hits");

        assert!(hits.get("initialize").is_some());
        assert_eq!(hits.get("initialize").unwrap().to_owned(), 1);
    }

    #[tokio::test]
    async fn test_notify() {
        let proxy = MockLspServerProxy::new();

        let client = Client::new(&proxy);

        let response = client.notify::<Exit>(());

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

    #[tokio::test]
    async fn test_concurrent() {
        let proxy = MockLspServerProxy::new();

        let client = Client::new(&proxy);

        let response = client.notify::<Exit>(());

        assert!(response.is_ok());

        let response_1 = client.request::<Initialize>(InitializeParams::default(), 1);
        let response_2 = client.request::<Initialize>(InitializeParams::default(), 2);

        let (first, second) = tokio::join!(response_1, response_2);

        assert!(first.is_ok());
        assert!(second.is_ok());

        assert_eq!(InitializeResult::default(), first.unwrap());
        assert_eq!(InitializeResult::default(), second.unwrap());

        let hits = proxy.hits.lock().expect("failed to acquire proxy hits");
        assert!(hits.get("initialize").is_some());
        assert_eq!(hits.get("initialize").unwrap().to_owned(), 2);
    }
}

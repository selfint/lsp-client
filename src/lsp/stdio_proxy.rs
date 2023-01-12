use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Result;
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::process::ChildStdin;
use tokio::process::ChildStdout;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

use crate::lsp::server_proxy::{LspServerProxy, ToServerChannel};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::lsp::protocol::{deserialize, serialize};

type Responses = Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>;
pub struct StdIOProxy {
    to_server: ToServerChannel,
    responses: Responses,
}

impl StdIOProxy {
    pub fn new(server_proc: tokio::process::Child) -> Self {
        let (to_server, receiver) = mpsc::unbounded_channel();
        let responses: Responses = Arc::new(Mutex::new(HashMap::new()));

        start_worker_threads(server_proc, receiver, &responses);

        Self {
            to_server,
            responses,
        }
    }
}

fn start_worker_threads(
    mut server_proc: tokio::process::Child,
    to_server_receiver: UnboundedReceiver<(Value, Option<oneshot::Sender<Value>>)>,
    responses: &Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
) {
    let proc_stdin = server_proc
        .stdin
        .take()
        .expect("failed to acquire process stdin");
    let proc_stdout = server_proc
        .stdout
        .take()
        .expect("failed to acquire process stdout");
    let proc_stderr = server_proc
        .stderr
        .take()
        .expect("failed to acquire process stderr");

    let (to_proc, to_proc_receiver) = mpsc::unbounded_channel();

    start_to_proc_thread(to_server_receiver, to_proc, Arc::clone(responses));
    start_to_stdin_thread(to_proc_receiver, proc_stdin);
    start_std_responses_thread(proc_stdout, Arc::clone(responses));
    start_std_responses_thread(proc_stderr, Arc::clone(responses));
}

/// Get outputs from process stdout/stderr and send results using the response channels.
fn start_std_responses_thread<T>(
    mut proc_stdout: T,
    responses: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
) where
    T: AsyncRead + std::marker::Unpin + std::marker::Send + 'static,
{
    tokio::spawn(async move {
        let mut buf = vec![];
        loop {
            let byte = proc_stdout
                .read_u8()
                .await
                .expect("failed to read from process stdout");

            buf.push(byte);

            let Ok(msg) = deserialize(&buf) else { continue };
            // we only need to send a response if the message has an id
            let Some(id) = msg.get("id") else { continue };
            let Some(id) = id.as_u64() else {panic!("failed to convert id '{:?}' to u64", id)};

            // get response channel, if none were found then we are done
            let Some(response_channel) = responses
                                .lock()
                                .expect("failed to acquire responses map")
                                .remove(&id) else { continue };

            response_channel.send(msg).unwrap_or_else(|_| {
                panic!(
                    "failed to send msg with id '{}' to its response channel",
                    id
                )
            });
        }
    });
}

/// Get inputs from input channel and send them to process stdin.
fn start_to_stdin_thread(
    mut to_proc_receiver: UnboundedReceiver<Vec<u8>>,
    mut proc_stdin: ChildStdin,
) {
    tokio::spawn(async move {
        while let Some(msg) = to_proc_receiver.recv().await {
            proc_stdin
                .write_all(&msg)
                .await
                .expect("failed to write msg to process stdin");
        }
    });
}

/// Send messages from to_server channel to process input channel.
/// Register the response channel for each message, if one is given.
fn start_to_proc_thread(
    mut to_server_receiver: UnboundedReceiver<(Value, Option<oneshot::Sender<Value>>)>,
    to_proc: UnboundedSender<Vec<u8>>,
    responses: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
) {
    tokio::spawn(async move {
        while let Some((msg, response_channel)) = to_server_receiver.recv().await {
            let serialized_msg =
                serialize(&msg).unwrap_or_else(|_| panic!("failed to serialize msg: {:?}", msg));

            to_proc.send(serialized_msg).expect("failed to send msg");

            if let Some(response_channel) = response_channel {
                let id = msg
                    .get("id")
                    .expect("got message with response channel without id")
                    .as_u64()
                    .expect("got non-u64 id");

                responses
                    .lock()
                    .expect("failed to acquire responses map")
                    .insert(id, response_channel);
            }
        }
    });
}

impl LspServerProxy for StdIOProxy {
    fn get_channel(&self) -> ToServerChannel {
        self.to_server.clone()
    }
}

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::process::ChildStderr;
use tokio::process::ChildStdin;
use tokio::process::ChildStdout;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

use serde_json::Value;

use crate::lsp::protocol::{deserialize, serialize};
use crate::lsp::server_proxy::{LspServerProxy, ToServerChannel};

pub struct StdIOProxy {
    to_server: ToServerChannel,
}

impl LspServerProxy for StdIOProxy {
    fn get_channel(&self) -> ToServerChannel {
        self.to_server.clone()
    }
}

impl StdIOProxy {
    pub fn new(stdin: ChildStdin, stdout: ChildStdout, stderr: ChildStderr) -> Self {
        let (to_server, receiver) = mpsc::unbounded_channel();
        let responses = Arc::new(Mutex::new(HashMap::new()));

        start_worker_threads(stdin, stdout, stderr, receiver, &responses);

        Self { to_server }
    }
}

fn start_worker_threads(
    stdin: ChildStdin,
    stdout: ChildStdout,
    stderr: ChildStderr,
    to_server_receiver: UnboundedReceiver<(Value, Option<oneshot::Sender<Value>>)>,
    responses: &Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
) {
    let (to_proc, to_proc_receiver) = mpsc::unbounded_channel();

    start_to_proc_thread(to_server_receiver, to_proc, Arc::clone(responses));
    start_to_stdin_thread(to_proc_receiver, stdin);
    start_stdout_responses_thread(stdout, Arc::clone(responses));
    start_stderr_responses_thread(stderr);
}

/// Log outputs from process stderr
fn start_stderr_responses_thread(mut proc_stderr: ChildStderr) {
    tokio::spawn(async move {
        let mut buf = vec![];
        loop {
            let byte = proc_stderr
                .read_u8()
                .await
                .expect("failed to read from process stderr");

            buf.push(byte);

            let text = std::str::from_utf8(&buf).unwrap();
            eprintln!("Got error: {}", text);

            if text.ends_with('\n') {
                buf.clear();
            }
        }
    });
}

/// Get outputs from process stdout and send results using the response channels.
fn start_stdout_responses_thread(
    mut proc_stdout: ChildStdout,
    responses: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
) {
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

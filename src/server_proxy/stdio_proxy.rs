use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::{ChildStderr, ChildStdin, ChildStdout},
    sync::{
        mpsc::{self, UnboundedReceiver},
        oneshot,
    },
};

use serde_json::Value;

use crate::{
    lsp_protocol::{deserialize, serialize},
    server_proxy::{LspServerProxy, ToServerChannel, ToServerMsg},
};

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
        let (to_server, to_server_receiver) = mpsc::unbounded_channel();

        let responses = Arc::new(Mutex::new(HashMap::new()));
        start_to_proc_thread(to_server_receiver, stdin, Arc::clone(&responses));
        start_stdout_responses_thread(stdout, responses);
        start_stderr_responses_thread(stderr);

        Self { to_server }
    }
}

/// Log outputs from process stderr
fn start_stderr_responses_thread(mut stderr: ChildStderr) {
    tokio::spawn(async move {
        let mut buf = vec![];
        loop {
            let byte = stderr
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
    mut stdout: ChildStdout,
    responses: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
) {
    tokio::spawn(async move {
        let mut buf = vec![];
        while let Ok(byte) = stdout.read_u8().await {
            buf.push(byte);

            let Ok(msg) = deserialize(&buf) else { continue };
            buf.clear();

            // we only need to send a response if the message has an id
            let Some(id) = msg.get("id") else { continue };
            let id = id.as_u64().expect("got non-u64 id");

            // get response channel, if none were found then we are done
            let mut responses_map = responses.lock().expect("failed to acquire responses map");
            let Some(response_channel) = responses_map.remove(&id) else { continue };

            let sent = response_channel.send(msg);
            if sent.is_err() {
                panic!(
                    "failed to send msg with id '{}' to its response channel",
                    id
                );
            };
        }

        panic!("failed to read from process stdout");
    });
}

/// Send messages from to_server channel to process input channel.
/// Register the response channel for each message, if one is given.
fn start_to_proc_thread(
    mut to_server_receiver: UnboundedReceiver<ToServerMsg>,
    mut stdin: ChildStdin,
    responses: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
) {
    tokio::spawn(async move {
        while let Some((msg, response_channel)) = to_server_receiver.recv().await {
            let serialized_msg =
                serialize(&msg).unwrap_or_else(|_| panic!("failed to serialize msg: {:?}", msg));

            stdin
                .write_all(&serialized_msg)
                .await
                .expect("failed to write msg to process stdin");

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

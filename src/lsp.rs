use std::io::{Read, Write};

use anyhow::Result;

use serde_json::Value;

use crate::jsonrpc::get_jsonrpc_content_length;

pub struct LspClient {
    send: std::sync::mpsc::Sender<Vec<u8>>,
    recv: std::sync::mpsc::Receiver<Vec<u8>>,
}

fn get_response<R, I, O, E>(
    msg: &[u8],
    input: &mut I,
    output: &mut O,
    error: &mut E,
) -> Result<Result<R::Result, Value>>
where
    R: lsp_types::request::Request,
    I: Write,
    O: Read + Send,
    E: Read + Send,
{
    input.write_all(msg)?;
    input.flush()?;

    let (sender_1, receiver) = std::sync::mpsc::channel::<Result<Value, Value>>();
    let sender_2 = sender_1.clone();

    std::thread::scope(|s| {
        s.spawn(move || {
            let content_length = '_get_content_length: {
                let mut buf = vec![];

                loop {
                    let byte = '_get_next_byte: {
                        let mut byte_buf: [u8; 1] = [0; 1];
                        output
                            .read_exact(&mut byte_buf)
                            .expect("failed to read from output");
                        byte_buf[0]
                    };

                    buf.push(byte);

                    if let Some(content_length) = get_jsonrpc_content_length(&buf) {
                        break content_length;
                    }
                }
            };

            let response = '_get_response: {
                let mut response_buf = vec![0; content_length];
                output
                    .read_exact(&mut response_buf)
                    .expect("failed to read from output");

                let Ok(response) = serde_json::from_slice::<Value>(&response_buf) else {
                    panic!("failed to deserialize response");
                };

                response
            };

            sender_1
                .send(Err(response))
                .expect("failed to send response");
        });

        s.spawn(move || {
            let mut buf = vec![];
            for byte in error.bytes() {
                let byte = byte.unwrap();
                buf.push(byte);

                if let Ok(msg) = serde_json::from_slice::<Value>(&buf) {
                    sender_2
                        .send(Err(msg))
                        .expect("failed to send response to from_server queue");

                    buf.clear();
                }
            }
        });

        let response = receiver
            .recv()
            .expect("failed to receive response")
            .expect("a");

        if let Ok(result) = serde_json::from_value::<R::Result>(response.clone()) {
            Ok(Ok(result))
        } else {
            Ok(Err(response))
        }
    })
}

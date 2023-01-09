use std::process::Stdio;

use lsp_client::LspClient;
use tokio::process::{Child, Command};

fn start_server() -> Child {
    Command::new("rust-analyzer")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start rust-analyzer")
}

#[tokio::test]
async fn test_connect_to_rust_analyzer() {
    let server = start_server();

    let _client = LspClient::new(server);
}

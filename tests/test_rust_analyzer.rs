use lsp_client::jsonrpc;
use lsp_types::request::Initialize;
use lsp_types::{InitializeParams, InitializeResult};
use tokio::process::Child;

use lsp_client::lsp::client::LspClient;
use lsp_client::lsp::server_proxy::proxies::stdio_proxy::StdIOProxy;

fn start_rust_analyzer() -> Child {
    tokio::process::Command::new("rust-analyzer")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to start rust analyzer")
}

#[tokio::test]
async fn test_rust_analyzer() {
    let mut server_proc = start_rust_analyzer();
    let stdin = server_proc
        .stdin
        .take()
        .expect("failed to acquire process stdin");
    let stdout = server_proc
        .stdout
        .take()
        .expect("failed to acquire process stdout");
    let stderr = server_proc
        .stderr
        .take()
        .expect("failed to acquire process stderr");

    let proxy = StdIOProxy::new(stdin, stdout, stderr);

    let client = LspClient::new(&proxy);

    let response = client
        .request::<Initialize, ()>(InitializeParams::default(), 0)
        .await;

    assert!(response.is_ok(), "{:?}", response);
    assert!(matches!(
        response.unwrap().result,
        jsonrpc::types::JsonRPCResult::Result(InitializeResult { .. })
    ));
}

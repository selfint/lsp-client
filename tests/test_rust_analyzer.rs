use std::time::Duration;

use lsp_types::notification::Initialized;
use lsp_types::request::Initialize;
use lsp_types::request::WorkspaceSymbol;
use lsp_types::InitializeError;
use lsp_types::InitializeParams;
use lsp_types::InitializeResult;
use lsp_types::InitializedParams;
use lsp_types::WorkspaceSymbolParams;

use lsp_client::jsonrpc;
use lsp_client::lsp::client::LspClient;
use lsp_client::lsp::proxies::stdio_proxy::StdIOProxy;

fn start_client() -> LspClient {
    let mut server_proc = tokio::process::Command::new("rust-analyzer")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to start rust analyzer");

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

    LspClient::new(&proxy)
}

#[tokio::test]
async fn test_rust_analyzer() {
    let client = start_client();

    let response = client
        .request::<Initialize, InitializeError>(InitializeParams::default(), 0)
        .await;

    assert!(response.is_ok(), "{:?}", response);
    assert!(matches!(
        response.unwrap().result,
        jsonrpc::types::JsonRPCResult::Result(InitializeResult { .. })
    ));

    let response = client.notify::<Initialized>(InitializedParams {});
    assert!(response.is_ok());

    let response = client
        .request::<WorkspaceSymbol, ()>(
            WorkspaceSymbolParams {
                query: " ".to_string(),
                ..Default::default()
            },
            1,
        )
        .await;

    assert!(response.is_ok());
    let mut response = response.unwrap();
    let mut id = 1;

    while let jsonrpc::types::JsonRPCResult::Error(ref result) = response.result {
        if result.code != lsp_types::error_codes::CONTENT_MODIFIED {
            panic!("Got error unexpected code: {:?}", result);
        }

        std::thread::sleep(Duration::from_millis(100));

        id += 1;
        response = client
            .request::<WorkspaceSymbol, ()>(
                WorkspaceSymbolParams {
                    query: " ".to_string(),
                    ..Default::default()
                },
                id,
            )
            .await
            .unwrap();
    }

    assert!(
        matches!(
            response.result,
            jsonrpc::types::JsonRPCResult::Result(Some(..))
        ),
        "{:?}",
        response
    )
}

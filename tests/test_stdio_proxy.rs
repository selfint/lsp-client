use std::time::Duration;

use lsp_client::{
    jsonrpc_types::JsonRPCResult,
    lsp_types::{
        notification::Initialized,
        request::{Initialize, Shutdown, WorkspaceSymbol},
        InitializeError, InitializeParams, InitializeResult, InitializedParams,
        WorkspaceSymbolParams,
    },
    server_proxy::StdIOProxy,
    LspClient,
};

use similar_asserts::assert_eq;

#[tokio::test]
async fn test_rust_analyzer() {
    let client = start_client(start_rust_analyzer());

    test_client(client).await;
}

#[tokio::test]
async fn test_jdtls() {
    let client = start_client(start_jdtls());

    test_client(client).await;
}

fn start_client(mut server_proc: tokio::process::Child) -> LspClient {
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

fn start_rust_analyzer() -> tokio::process::Child {
    tokio::process::Command::new("rust-analyzer")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to start rust analyzer")
}

fn start_jdtls() -> tokio::process::Child {
    tokio::process::Command::new("jdtls")
        .arg("-data")
        .arg(std::env::temp_dir().to_str().unwrap())
        .arg("-configuration")
        .arg(std::env::temp_dir().to_str().unwrap())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to start jdtls")
}

async fn test_client(client: LspClient) {
    let response = client
        .request::<Initialize, InitializeError>(InitializeParams::default(), 0)
        .await;

    assert!(response.is_ok(), "{:?}", response);
    assert!(matches!(
        response.unwrap().result,
        JsonRPCResult::Result(InitializeResult { .. })
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

    while let JsonRPCResult::Error(ref result) = response.result {
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
        matches!(response.result, JsonRPCResult::Result(Some(..))),
        "{:?}",
        response
    );

    let response = client.request::<Shutdown, ()>((), id + 1).await;

    assert!(response.is_ok());
    assert_eq!(JsonRPCResult::Result(()), response.unwrap().result);
}

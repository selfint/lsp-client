use anyhow::Ok;
use anyhow::Result;
use jsonrpsee_types::Id;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;

lazy_static! {
    static ref JSONRPC_HEADER_RE: Regex = Regex::new(&{
        use normal::prelude::*;

        "Content-Length: "
            .then_named_group("contentLength", at_least_once(DIGIT))
            .then_optional(
                CARRIAGE_RETURN
                    .then(NEWLINE)
                    .then("Content-Type: ")
                    .then_repeated(r"[^\r]"),
            )
            .then(CARRIAGE_RETURN)
            .then(NEWLINE)
            .then(CARRIAGE_RETURN)
            .then(NEWLINE)
    })
    .unwrap();
}

pub fn get_jsonrpc_content_length(buf: &[u8]) -> Option<usize> {
    let text = &std::str::from_utf8(buf).expect("failed to convert buffer to utf-8");

    if let Some(matches) = JSONRPC_HEADER_RE.clone().captures(text) {
        let first_match = matches
            .name("contentLength")
            .expect("failed to extract content-length");

        Some(
            first_match
                .as_str()
                .parse::<usize>()
                .expect("failed to parse content-length"),
        )
    } else {
        None
    }
}

fn build_jsonrpc_request<P: Serialize>(method: &str, params: &Option<P>, id: Id) -> Result<String> {
    let params = match params {
        Some(params) => Some(serde_json::value::to_raw_value(params)?),
        None => None,
    };

    let jsonrpc_request = jsonrpsee_types::request::RequestSer::owned(id, method, params);
    let jsonrpc_request = serde_json::to_string(&jsonrpc_request)?;

    Ok(jsonrpc_request)
}

fn add_jsonrpc_header(msg: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{}", msg.len(), msg)
}

#[cfg(test)]
mod tests {
    use jsonrpsee_types::Request;

    use super::*;

    #[test]
    fn test_build_jsonrpc_request() {
        #[derive(Serialize)]
        struct Params {
            minuend: u32,
            subtrahead: u32,
        }

        let params = serde_json::value::to_raw_value(&Params {
            minuend: 42,
            subtrahead: 23,
        })
        .unwrap();

        let expected = Request {
            jsonrpc: jsonrpsee_types::TwoPointZero,
            id: Id::Number(3),
            method: "subtract".into(),
            params: Some(&params),
        };

        let actual = build_jsonrpc_request(
            "subtract",
            &Some(Params {
                minuend: 42,
                subtrahead: 23,
            }),
            Id::Number(3),
        );

        assert!(actual.is_ok());
        assert_requests_eq(expected, actual.unwrap())
    }

    #[test]
    fn test_build_jsonrpc_request_no_params() {
        let expected = Request {
            jsonrpc: jsonrpsee_types::TwoPointZero,
            id: Id::Number(3),
            method: "subtract".into(),
            params: None,
        };

        let actual = build_jsonrpc_request::<()>("subtract", &None, Id::Number(3));

        assert!(actual.is_ok());
        assert_requests_eq(expected, actual.unwrap())
    }

    fn assert_requests_eq(expected: Request, actual: String) {
        let actual = serde_json::from_str::<Request>(&actual);

        assert!(actual.is_ok());
        assert_eq!(
            serde_json::to_string(&expected).unwrap(),
            serde_json::to_string(&actual.unwrap()).unwrap()
        );
    }

    #[test]
    fn test_add_jsonrpc_header() {
        assert_eq!("Content-Length: 1\r\n\r\na", add_jsonrpc_header("a"));
    }
}

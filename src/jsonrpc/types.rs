use serde::{Deserialize, Serialize};

pub const JSONRPC_V2: &str = "2.0";

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
struct Request<P> {
    jsonrpc: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<P>,
    id: Option<usize>,
}

impl<P> Request<P> {
    fn new(method: impl Into<String>, params: Option<P>, id: Option<usize>) -> Self {
        Self {
            jsonrpc: JSONRPC_V2.to_string(),
            method: method.into(),
            params,
            id,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
struct Response<R, E> {
    jsonrpc: String,
    result: JsonRPCResult<R, E>,
    id: Option<usize>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
enum JsonRPCResult<R, E> {
    Ok(R),
    Err(JsonRPCError<E>),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
struct JsonRPCError<D> {
    code: i32,
    message: String,
    data: D,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
struct Notification<P> {
    jsonrpc: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<P>,
}

#[cfg(test)]
mod tests {
    use serde::de::DeserializeOwned;
    use serde_json::{json, Value};

    use super::*;

    fn check_serde<D>(expected_serialized: Value, expected_deserialized: D)
    where
        D: Serialize + DeserializeOwned + PartialEq + Eq + std::fmt::Debug,
    {
        let actual_deserialized = serde_json::from_value::<D>(expected_serialized.clone());
        let actual_serialized = serde_json::to_value(&expected_deserialized);

        assert!(actual_deserialized.is_ok());
        assert_eq!(actual_deserialized.unwrap(), expected_deserialized);

        assert!(actual_serialized.is_ok());
        assert_eq!(actual_serialized.unwrap(), expected_serialized);
    }

    #[test]
    fn test_request_serde() {
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
        struct Params {
            p0: u32,
            p1: u32,
        }

        check_serde(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
                "params": {"p0": 0, "p1": 1},
                "id": 1
            }),
            Request::new("method", Some(Params { p0: 0, p1: 1 }), Some(1)),
        );
        check_serde(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
                "id": 1
            }),
            Request::<()>::new("method", None, Some(1)),
        );
        check_serde(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
                "id": null
            }),
            Request::<()>::new("method", None, None),
        );
    }

    #[test]
    fn test_notification_serde() {
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
        struct Params {
            p0: u32,
            p1: u32,
        }

        check_serde(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
                "params": {"p0": 0, "p1": 1},
                "id": 1
            }),
            Request::new("method", Some(Params { p0: 0, p1: 1 }), Some(1)),
        );
        check_serde(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
                "id": 1
            }),
            Request::<()>::new("method", None, Some(1)),
        );
        check_serde(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
                "id": null,
            }),
            Request::<()>::new("method", None, None),
        );
    }
}

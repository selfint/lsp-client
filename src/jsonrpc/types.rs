use serde::{Deserialize, Serialize};

pub const JSONRPC_V2: &str = "2.0";

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct Request<P> {
    jsonrpc: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<P>,
    id: Option<usize>,
}

impl<P> Request<P> {
    pub fn new(method: impl Into<String>, params: Option<P>, id: Option<usize>) -> Self {
        Self {
            jsonrpc: JSONRPC_V2.to_string(),
            method: method.into(),
            params,
            id,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct Response<R, E> {
    jsonrpc: String,
    #[serde(flatten)]
    result: JsonRPCResult<R, E>,
    id: Option<usize>,
}

impl<R, E> Response<R, E> {
    pub fn new(result: JsonRPCResult<R, E>, id: Option<usize>) -> Self {
        Self {
            jsonrpc: JSONRPC_V2.to_string(),
            result,
            id,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "lowercase")]
pub enum JsonRPCResult<R, E> {
    Result(R),
    Error(JsonRPCError<E>),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct JsonRPCError<D> {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<D>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct Notification<P> {
    jsonrpc: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<P>,
}

impl<P> Notification<P> {
    pub fn new(method: impl Into<String>, params: Option<P>) -> Self {
        Self {
            jsonrpc: JSONRPC_V2.to_string(),
            method: method.into(),
            params,
        }
    }
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
        let actual_serialized = serde_json::to_value(&expected_deserialized);

        assert!(actual_serialized.is_ok());
        assert_eq!(expected_serialized, actual_serialized.unwrap());

        let actual_deserialized = serde_json::from_value::<D>(expected_serialized);

        assert!(actual_deserialized.is_ok());
        assert_eq!(expected_deserialized, actual_deserialized.unwrap());
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
                "id": 2
            }),
            Request::<()>::new("method", None, Some(2)),
        );
        check_serde(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
                "id": null
            }),
            Request::<()>::new("method", None, None),
        );
        check_serde(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
                "params": [0, 1],
                "id": 3
            }),
            Request::new("method", Some(vec![0, 1]), Some(3)),
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
            }),
            Notification::new("method", Some(Params { p0: 0, p1: 1 })),
        );
        check_serde(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
            }),
            Notification::<()>::new("method", None),
        );
        check_serde(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
                "params": [0, 1],
            }),
            Notification::new("method", Some(vec![0, 1])),
        );
    }

    #[test]
    fn test_response_serde() {
        check_serde(
            json!({
                "jsonrpc": "2.0",
                "result": 1,
                "id": 1
            }),
            Response::new(JsonRPCResult::<u32, ()>::Result(1), Some(1)),
        );
        check_serde(
            json!({
                "jsonrpc": "2.0",
                "result": 1,
                "id": null
            }),
            Response::new(JsonRPCResult::<u32, ()>::Result(1), None),
        );

        check_serde(
            json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -1,
                    "message": "message"
                },
                "id": 2
            }),
            Response::new(
                JsonRPCResult::<(), ()>::Error(JsonRPCError {
                    code: -1,
                    message: "message".to_string(),
                    data: None,
                }),
                Some(2),
            ),
        );

        #[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
        struct ErrorData {
            error_data: String,
        }

        check_serde(
            json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -1,
                    "message": "message",
                    "data": {
                        "error_data": "error_data"
                    }
                },
                "id": 3
            }),
            Response::new(
                JsonRPCResult::<(), ErrorData>::Error(JsonRPCError {
                    code: -1,
                    message: "message".to_string(),
                    data: Some(ErrorData {
                        error_data: "error_data".to_string(),
                    }),
                }),
                Some(3),
            ),
        );
    }
}

use serde::{Deserialize, Serialize};

pub const JSONRPC_V2: &str = "2.0";

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Request<P> {
    pub jsonrpc: String,
    pub method: String,
    pub params: P,
    pub id: Option<u64>,
}

impl<P> Request<P> {
    pub fn new(method: impl Into<String>, params: P, id: Option<u64>) -> Self {
        Self {
            jsonrpc: JSONRPC_V2.to_string(),
            method: method.into(),
            params,
            id,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Response<R, E> {
    pub jsonrpc: String,
    #[serde(flatten)]
    pub result: JsonRPCResult<R, E>,
    pub id: Option<u64>,
}

impl<R, E> Response<R, E> {
    pub fn new(result: JsonRPCResult<R, E>, id: Option<u64>) -> Self {
        Self {
            jsonrpc: JSONRPC_V2.to_string(),
            result,
            id,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum JsonRPCResult<R, E> {
    Result(R),
    Error(JsonRPCError<E>),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct JsonRPCError<D> {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<D>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Notification<P> {
    pub jsonrpc: String,
    pub method: String,
    pub params: P,
}

impl<P> Notification<P> {
    pub fn new(method: impl Into<String>, params: P) -> Self {
        Self {
            jsonrpc: JSONRPC_V2.to_string(),
            method: method.into(),
            params,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    macro_rules! assert_serde {
        ($expected_serialized:expr, $expected_deserialized:expr, $ty:ty) => {
            let actual_serialized = serde_json::to_value(&$expected_deserialized);

            assert!(actual_serialized.is_ok());
            assert_eq!($expected_serialized, actual_serialized.unwrap());

            let actual_deserialized = serde_json::from_value::<$ty>($expected_serialized);

            assert!(actual_deserialized.is_ok());
            assert_eq!($expected_deserialized, actual_deserialized.unwrap());
        };
    }

    #[test]
    fn test_request_serde() {
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
        struct Params {
            p0: u32,
            p1: u32,
        }

        assert_serde!(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
                "params": {"p0": 0, "p1": 1},
                "id": 1
            }),
            Request::new("method", Params { p0: 0, p1: 1 }, Some(1)),
            Request<Params>
        );
        assert_serde!(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
                "params": null,
                "id": 2,
            }),
            Request::<()>::new("method", (), Some(2)),
            Request<()>
        );
        assert_serde!(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
                "params": null,
                "id": null,
            }),
            Request::<()>::new("method", (), None),
            Request<()>
        );
        assert_serde!(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
                "params": [0, 1],
                "id": 3
            }),
            Request::new("method", vec![0, 1], Some(3)),
            Request<Vec<i32>>
        );
        assert_serde!(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
                "params": null,
                "id": 4
            }),
            Request::<()>::new("method", (), Some(4)),
            Request<()>
        );
    }

    #[test]
    fn test_notification_serde() {
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
        struct Params {
            p0: u32,
            p1: u32,
        }

        assert_serde!(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
                "params": {"p0": 0, "p1": 1},
            }),
            Notification::new("method", Params { p0: 0, p1: 1 }),
            Notification<Params>
        );
        assert_serde!(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
                "params": null,
            }),
            Notification::<()>::new("method", ()),
            Notification<()>
        );
        assert_serde!(
            json!({
                "jsonrpc": "2.0",
                "method": "method",
                "params": [0, 1],
            }),
            Notification::new("method", vec![0, 1]),
            Notification<Vec<i32>>
        );
    }

    #[test]
    fn test_response_serde() {
        assert_serde!(
            json!({
                "jsonrpc": "2.0",
                "result": 1,
                "id": 1
            }),
            Response::new(JsonRPCResult::<u32, ()>::Result(1), Some(1)),
            Response<u32, ()>
        );
        assert_serde!(
            json!({
                "jsonrpc": "2.0",
                "result": 1,
                "id": null
            }),
            Response::new(JsonRPCResult::<u32, ()>::Result(1), None),
            Response<u32, ()>
        );
        assert_serde!(
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
            Response<(), ()>
        );

        #[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
        struct ErrorData {
            error_data: String,
        }

        assert_serde!(
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
            Response<(), ErrorData>
        );
    }
}

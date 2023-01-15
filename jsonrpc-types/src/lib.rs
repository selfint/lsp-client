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
    use similar_asserts::assert_eq;

    use super::*;

    macro_rules! snapshot {
        ($e:expr) => {
            insta::assert_json_snapshot!($e);

            let serialized = serde_json::to_value($e);
            let deserialized = serde_json::from_value(serialized.unwrap()).unwrap();

            assert_eq!($e, deserialized);
        };
    }

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct Params {
        p0: u32,
        p1: u32,
    }

    #[test]
    fn test_request_serde() {
        snapshot!(Request::new("method", (), None));
        snapshot!(Request::new("method", (), Some(1)));
        snapshot!(Request::new("method", vec![0, 1], None));
        snapshot!(Request::new("method", vec![0, 1], Some(1)));
        snapshot!(Request::new("method", Params { p0: 0, p1: 1 }, None));
        snapshot!(Request::new("method", Params { p0: 0, p1: 1 }, Some(1)));
    }

    #[test]
    fn test_notification_serde() {
        snapshot!(Notification::new("method", ()));
        snapshot!(Notification::new("method", vec![0, 1]));
        snapshot!(Notification::new("method", Params { p0: 0, p1: 1 }));
    }

    #[test]
    fn test_response_serde() {
        macro_rules! snapshot_permutations {
            ($data:expr) => {
                snapshot!(Response::new(
                    JsonRPCResult::<_, ()>::Result($data),
                    Some(1)
                ));
                snapshot!(Response::new(JsonRPCResult::<_, ()>::Result($data), None));
                snapshot!(Response::new(
                    JsonRPCResult::<(), _>::Error(JsonRPCError {
                        code: -1,
                        message: "message".to_string(),
                        data: Some($data)
                    }),
                    Some(1)
                ));
                snapshot!(Response::new(
                    JsonRPCResult::<(), ()>::Error(JsonRPCError {
                        code: -1,
                        message: "message".to_string(),
                        data: None
                    }),
                    Some(1)
                ));
                snapshot!(Response::new(
                    JsonRPCResult::<(), _>::Error(JsonRPCError {
                        code: -1,
                        message: "message".to_string(),
                        data: Some($data)
                    }),
                    None
                ));
                snapshot!(Response::new(
                    JsonRPCResult::<(), ()>::Error(JsonRPCError {
                        code: -1,
                        message: "message".to_string(),
                        data: None
                    }),
                    None
                ));
            };
        }

        snapshot_permutations!(1);
        snapshot_permutations!(vec![1, -1]);
        snapshot_permutations!(Params { p0: 0, p1: 1 });
    }
}

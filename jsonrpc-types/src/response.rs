use serde::{ser::SerializeStruct, Deserialize, Serialize};

use crate::JSONRPC_V2;

#[derive(Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Response<R, E> {
    #[serde(flatten)]
    pub result: JsonRPCResult<R, E>,
    pub id: Option<u64>,
}

impl<R, E> Response<R, E> {
    pub fn new(result: JsonRPCResult<R, E>, id: Option<u64>) -> Self {
        Self { result, id }
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

impl<R: Serialize, E: Serialize> Serialize for Response<R, E> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Response", 3)?;
        state.serialize_field("jsonrpc", JSONRPC_V2)?;

        // flatten result
        match &self.result {
            JsonRPCResult::Result(r) => state.serialize_field("result", r)?,
            JsonRPCResult::Error(e) => state.serialize_field("error", e)?,
        };

        state.serialize_field("id", &self.id)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::{snapshot, Params};

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

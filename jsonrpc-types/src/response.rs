use serde::{ser::SerializeStruct, Deserialize, Serialize};

use crate::JSONRPC_V2;

#[derive(Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Response<R, E> {
    #[serde(flatten)]
    pub content: ResponseContent<R, E>,
    pub id: Option<u64>,
}

impl<R, E> Response<R, E> {
    pub fn new(content: ResponseContent<R, E>, id: Option<u64>) -> Self {
        Self { content, id }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ResponseContent<R, E> {
    Result(R),
    Error(ResponseError<E>),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct ResponseError<D> {
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
        match &self.content {
            ResponseContent::Result(r) => state.serialize_field("result", r)?,
            ResponseContent::Error(e) => state.serialize_field("error", e)?,
        }

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
                    ResponseContent::<_, ()>::Result($data),
                    Some(1)
                ));
                snapshot!(Response::new(ResponseContent::<_, ()>::Result($data), None));
                snapshot!(Response::new(
                    ResponseContent::<(), _>::Error(ResponseError {
                        code: -1,
                        message: "message".to_string(),
                        data: Some($data)
                    }),
                    Some(1)
                ));
                snapshot!(Response::new(
                    ResponseContent::<(), ()>::Error(ResponseError {
                        code: -1,
                        message: "message".to_string(),
                        data: None
                    }),
                    Some(1)
                ));
                snapshot!(Response::new(
                    ResponseContent::<(), _>::Error(ResponseError {
                        code: -1,
                        message: "message".to_string(),
                        data: Some($data)
                    }),
                    None
                ));
                snapshot!(Response::new(
                    ResponseContent::<(), ()>::Error(ResponseError {
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

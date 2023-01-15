use serde::{ser::SerializeStruct, Deserialize, Serialize};

use crate::JSONRPC_V2;

#[derive(Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Request<P> {
    pub method: String,
    pub params: P,
    pub id: Option<u64>,
}

impl<P> Request<P> {
    pub fn new(method: impl Into<String>, params: P, id: Option<u64>) -> Self {
        Self {
            method: method.into(),
            params,
            id,
        }
    }
}

impl<P: Serialize> Serialize for Request<P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Request", 4)?;
        state.serialize_field("jsonrpc", JSONRPC_V2)?;
        state.serialize_field("method", &self.method)?;
        state.serialize_field("params", &self.params)?;
        state.serialize_field("id", &self.id)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::{snapshot, Params};

    #[test]
    fn test_request_serde() {
        snapshot!(Request::new("method", (), None));
        snapshot!(Request::new("method", (), Some(1)));
        snapshot!(Request::new("method", vec![0, 1], None));
        snapshot!(Request::new("method", vec![0, 1], Some(1)));
        snapshot!(Request::new("method", Params { p0: 0, p1: 1 }, None));
        snapshot!(Request::new("method", Params { p0: 0, p1: 1 }, Some(1)));
    }
}

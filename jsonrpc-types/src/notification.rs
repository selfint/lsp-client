use serde::{ser::SerializeStruct, Deserialize, Serialize};

use crate::JSONRPC_V2;

#[derive(Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Notification<P> {
    pub method: String,
    pub params: P,
}

impl<P> Notification<P> {
    pub fn new(method: impl Into<String>, params: P) -> Self {
        Self {
            method: method.into(),
            params,
        }
    }
}

impl<P: Serialize> Serialize for Notification<P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Notification", 3)?;
        state.serialize_field("jsonrpc", JSONRPC_V2)?;
        state.serialize_field("method", &self.method)?;
        state.serialize_field("params", &self.params)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::{snapshot, Params};

    #[test]
    fn test_notification_serde() {
        snapshot!(Notification::new("method", ()));
        snapshot!(Notification::new("method", vec![0, 1]));
        snapshot!(Notification::new("method", Params { p0: 0, p1: 1 }));
    }
}

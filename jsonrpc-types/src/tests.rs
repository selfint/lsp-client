use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub(crate) struct Params {
    pub p0: u32,
    pub p1: u32,
}

#[cfg(test)]
macro_rules! snapshot {
    ($e:expr) => {
        insta::assert_json_snapshot!($e);

        let serialized = serde_json::to_value($e);
        let deserialized = serde_json::from_value(serialized.unwrap()).unwrap();

        similar_asserts::assert_eq!($e, deserialized);
    };
}

#[cfg(test)]
pub(crate) use snapshot;

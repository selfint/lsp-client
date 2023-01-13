use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;

lazy_static! {
    static ref LSP_PACKET_RE: Regex = Regex::new(&{
        use normal::prelude::*;

        "Content-Length: "
            .then_named_group("length", at_least_once(DIGIT))
            .then_optional(group(
                r"\r\n".then("Content-Type: ").then_repeated(r"[^\r]"),
            ))
            .then(r"\r\n\r\n")
            .then_named_group("content", repeated("."))
    })
    .unwrap();
}

pub fn serialize(msg: &Value) -> Result<Vec<u8>> {
    let msg = serde_json::to_string(msg)?;
    let msg_len = msg.len();

    Ok(format!("Content-Length: {}\r\n\r\n{}", msg_len, msg)
        .as_bytes()
        .to_vec())
}

pub fn deserialize(msg: &[u8]) -> Result<Value> {
    let text = std::str::from_utf8(msg)?;

    let captures = LSP_PACKET_RE
        .captures(text)
        .context("failed to match lsp header")?;

    let length = captures
        .name("length")
        .context("failed to extract content length")?
        .as_str()
        .parse::<usize>()?;

    let content = captures
        .name("content")
        .context("failed to extract content")?
        .as_str();

    match content.len().cmp(&length) {
        std::cmp::Ordering::Less => None.context("missing content"),
        std::cmp::Ordering::Equal => Ok(serde_json::from_str(content)?),
        std::cmp::Ordering::Greater => panic!("received more content than expected length"),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_serde() {
        let expected_deserialized = json!({"a": "b"});
        let expected_serialized = "Content-Length: 9\r\n\r\n{\"a\":\"b\"}".as_bytes().to_vec();

        let actual_serialized = serialize(&expected_deserialized);

        assert!(actual_serialized.is_ok());
        assert_eq!(
            std::str::from_utf8(&expected_serialized),
            std::str::from_utf8(&actual_serialized.unwrap())
        );

        let actual_deserialized = deserialize(&expected_serialized);
        assert!(actual_deserialized.is_ok());

        assert_eq!(expected_deserialized, actual_deserialized.unwrap());
    }
}

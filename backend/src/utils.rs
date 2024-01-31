pub mod acls;

pub mod json {
    use serde_json::Value;
    use std::collections::BTreeMap;

    pub fn links_into_btree_map(links: Value) -> Result<BTreeMap<String, String>, String> {
        if let Value::Object(map) = links {
            map.into_iter()
                .map(|(k, v)| {
                    Ok((
                        k,
                        v.as_str().ok_or("links value is not a string")?.to_string(),
                    ))
                })
                .collect::<Result<_, _>>()
        } else {
            Err(format!("links value ({links}) is not an object"))
        }
    }

    pub fn metadata_into_btree_map(metadata: Value) -> Result<BTreeMap<String, Value>, String> {
        if let Value::Object(map) = metadata {
            Ok(map.into_iter().collect())
        } else {
            Err("metadata is not an object".to_string())
        }
    }
}

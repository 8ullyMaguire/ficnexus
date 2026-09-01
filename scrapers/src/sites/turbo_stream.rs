//! Turbo-stream decoder (the compact serialization SoFurry's `.data`
//! endpoints use).
//!
//! The payload is a JSON array ("slot table"); every entry is either a
//! plain value, a slot reference (integer), an object whose keys/values
//! are slot references, an array of slot references, or a tagged value
//! like `["D", "..."]`. This module resolves the table into a plain
//! [`serde_json::Value`].

use serde_json::{Map, Value};

/// Decode a turbo-stream body (first line only; later lines stream extra
/// data and are ignored). Returns the resolved root value.
pub fn decode(payload: &str) -> Result<Value, String> {
    let first_line = payload.split('\n').next().unwrap_or("");
    let table: Vec<Value> =
        serde_json::from_str(first_line).map_err(|e| format!("turbo-stream: {e}"))?;

    let mut done: Vec<Option<Value>> = vec![None; table.len()];

    fn resolve(table: &[Value], done: &mut [Option<Value>], slot: i64) -> Result<Value, String> {
        // Negative slots are sentinels (literal values).
        match slot {
            -1 | -5 | -7 => return Ok(Value::Null),
            -2 => return Ok(Value::Null), // NaN → null
            -3 => return Ok(Value::Null), // -inf → null
            -4 => return Ok(Value::Number(serde_json::Number::from_f64(-0.0).unwrap_or(0.into()))),
            -6 => return Ok(Value::Null), // +inf → null
            _ => {}
        }
        if slot < 0 || slot as usize >= table.len() {
            return Err(format!("turbo-stream: bad slot {slot}"));
        }
        let idx = slot as usize;
        if let Some(v) = &done[idx] {
            return Ok(v.clone());
        }
        let entry = table[idx].clone();
        let resolved = resolve_entry(table, done, entry)?;
        done[idx] = Some(resolved.clone());
        Ok(resolved)
    }

    fn resolve_entry(
        table: &[Value],
        done: &mut [Option<Value>],
        entry: Value,
    ) -> Result<Value, String> {
        match entry {
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    // Integer could be a slot reference OR a plain number.
                    // Turbo-stream disambiguates: plain numbers are stored as
                    // strings in the tagged format; bare integers are refs.
                    resolve(table, done, i)
                } else {
                    Ok(Value::Number(n))
                }
            }
            Value::String(_) | Value::Bool(_) | Value::Null => Ok(entry),
            Value::Array(items) => {
                if let Some(Value::String(tag)) = items.first() {
                    resolve_tagged(table, done, tag, &items[1..])
                } else {
                    let mut out = Vec::with_capacity(items.len());
                    for item in items {
                        let slot = item
                            .as_i64()
                            .ok_or_else(|| format!("turbo-stream: array item not a slot: {item}"))?;
                        out.push(resolve(table, done, slot)?);
                    }
                    Ok(Value::Array(out))
                }
            }
            Value::Object(map) => {
                let mut out = Map::new();
                for (k, v) in map {
                    let key_slot = k
                        .strip_prefix('_')
                        .ok_or_else(|| format!("turbo-stream: bad key {k}"))?
                        .parse::<i64>()
                        .map_err(|e| format!("turbo-stream: bad key {k}: {e}"))?;
                    let key = resolve(table, done, key_slot)?;
                    let key = key
                        .as_str()
                        .ok_or_else(|| format!("turbo-stream: key not string: {key}"))?
                        .to_string();
                    let val_slot = v
                        .as_i64()
                        .ok_or_else(|| format!("turbo-stream: value not a slot: {v}"))?;
                    let val = resolve(table, done, val_slot)?;
                    out.insert(key, val);
                }
                Ok(Value::Object(out))
            }
        }
    }

    fn resolve_tagged(
        table: &[Value],
        done: &mut [Option<Value>],
        tag: &str,
        args: &[Value],
    ) -> Result<Value, String> {
        let slot_of = |v: &Value| {
            v.as_i64().ok_or_else(|| format!("turbo-stream: bad {tag} arg {v}"))
        };
        match tag {
            "D" | "U" | "Y" => Ok(args
                .first()
                .cloned()
                .unwrap_or(Value::Null)),
            "B" => Ok(args
                .first()
                .and_then(|a| a.as_i64())
                .map(|i| Value::Number(i.into()))
                .unwrap_or(Value::Null)),
            "R" => {
                // RegExp → {"pattern":..,"flags":..} (rarely needed)
                Ok(Value::Null)
            }
            "S" => {
                let mut out = Vec::with_capacity(args.len());
                for a in args {
                    out.push(resolve(table, done, slot_of(a)?)?);
                }
                Ok(Value::Array(out))
            }
            "M" => {
                let mut out = Map::new();
                for pair in args.chunks(2) {
                    if pair.len() == 2 {
                        let k = resolve(table, done, slot_of(&pair[0])?)?;
                        let k = k.as_str().unwrap_or("").to_string();
                        let v = resolve(table, done, slot_of(&pair[1])?)?;
                        out.insert(k, v);
                    }
                }
                Ok(Value::Object(out))
            }
            "N" => {
                let mut out = Map::new();
                if let Some(Value::Object(map)) = args.first() {
                    for (k, v) in map {
                        let key_slot = k
                            .strip_prefix('_')
                            .ok_or_else(|| format!("turbo-stream: bad N key {k}"))?
                            .parse::<i64>()
                            .map_err(|e| format!("turbo-stream: bad N key {k}: {e}"))?;
                        let key = resolve(table, done, key_slot)?;
                        let key = key.as_str().unwrap_or("").to_string();
                        let val = resolve(table, done, slot_of(v)?)?;
                        out.insert(key, val);
                    }
                }
                Ok(Value::Object(out))
            }
            "Z" => {
                let target = args
                    .first()
                    .and_then(|a| a.as_i64())
                    .ok_or_else(|| "turbo-stream: bad Z arg".to_string())?;
                resolve(table, done, target)
            }
            other => Err(format!("turbo-stream: unknown tag {other:?}")),
        }
    }

    // Root is slot 0 (the whole payload is a slot table; slot 0 is the
    // top-level value, usually an object keyed by route names).
    if table.is_empty() {
        return Err("turbo-stream: empty".into());
    }
    let root = resolve(&table, &mut done, 0)?;
    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_plain_values() {
        let payload = r#"["hello"]"#;
        let v = decode(payload).unwrap();
        assert_eq!(v, serde_json::json!("hello"));
    }

    #[test]
    fn decodes_slot_table() {
        // slot0 = {"_3": 4} → key slot 3 = "title", value slot 4 = "My Story"
        let payload = r#"[{"_3": 4}, "k1", "k2", "title", "My Story"]"#;
        let v = decode(payload).unwrap();
        assert_eq!(v, serde_json::json!({"title": "My Story"}));
    }

    #[test]
    fn decodes_tagged_date() {
        let payload = r#"[["D", "2024-01-31T10:00:00.000Z"]]"#;
        let v = decode(payload).unwrap();
        assert_eq!(v, serde_json::json!("2024-01-31T10:00:00.000Z"));
    }

    #[test]
    fn decodes_shared_refs() {
        // slot0 = {"_1": 4, "_2": 4} → key "x", value "v" (shared slot 4)
        let payload = r#"[{"_1": 4, "_2": 4}, "x", "y", "z", "v"]"#;
        let v = decode(payload).unwrap();
        assert_eq!(v, serde_json::json!({"x": "v", "y": "v"}));
    }
}

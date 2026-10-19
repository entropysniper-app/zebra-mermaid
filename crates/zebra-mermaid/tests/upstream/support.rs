//! Shared runtime for mechanically translated parser assertions, compiled inside the library tests.
//! Expected values and assertion operations come from unchanged upstream test callbacks.
use serde_json::{json, Value};
pub trait Adapter {
    fn reset(&mut self);
    fn parse(&mut self, source: &str) -> Result<(), String>;
    fn root(&self, key: &str) -> Result<Value, String>;
}
pub fn map(entries: impl IntoIterator<Item = (String, Value)>) -> Value {
    json!({"$map":entries.into_iter().map(|(k,v)|json!([k,v])).collect::<Vec<_>>()})
}
fn evaluate(e: &Value, db: &impl Adapter) -> Result<Option<Value>, String> {
    let at = |key: &str| e.get(key).ok_or_else(|| format!("missing IR field {key}"));
    let op = at("op")?.as_str().ok_or("IR op is not a string")?;
    Ok(match op {
        "undefined" => None,
        "literal" => Some(at("value")?.clone()),
        "root" => Some(db.root(at("key")?.as_str().ok_or("root key")?).expect("unsupported adapter root")),
        "array" => Some(Value::Array(
            at("items")?
                .as_array()
                .ok_or("IR items")?
                .iter()
                .map(|v| evaluate(v, db)?.ok_or_else(|| "undefined array entry requires explicit adapter".into()))
                .collect::<Result<_, String>>()?,
        )),
        "object" => {
            let mut result = serde_json::Map::new();
            for item in at("entries")?.as_array().ok_or("IR entries")? {
                let key = item[0].as_str().ok_or("IR object key")?;
                let value = evaluate(&item[1], db)?.ok_or("undefined object entry requires explicit adapter")?;
                result.insert(key.into(), value);
            }
            Some(Value::Object(result))
        }
        "map" => {
            let mut result = vec![];
            for item in at("entries")?.as_array().ok_or("IR entries")? {
                let key = evaluate(&item[0], db)?.ok_or("undefined map key")?;
                let val = evaluate(&item[1], db)?.ok_or("undefined map value")?;
                result.push(json!([key, val]));
            }
            Some(json!({"$map":result}))
        }
        "property" => {
            let base = evaluate(at("base")?, db)?.ok_or("Cannot read properties of undefined")?;
            let key = at("key")?.as_str().ok_or("property key")?;
            if base.is_null() {
                return Err("Cannot read properties of null".into());
            }
            if key == "size" && base.get("$map").is_some() {
                Some(json!(base["$map"].as_array().ok_or("map entries")?.len()))
            } else if key == "length" {
                if let Some(v) = base.as_array() {
                    Some(json!(v.len()))
                } else if let Some(v) = base.as_str() {
                    Some(json!(crate::util::js::len16(v)))
                } else {
                    base.get(key).cloned()
                }
            } else if let Some(array) = base.as_array() {
                key.parse::<usize>().ok().and_then(|i| array.get(i)).cloned()
            } else {
                base.get(key).cloned()
            }
        }
        "get" | "has" => {
            let base = evaluate(at("base")?, db)?.ok_or("Map operation on undefined")?;
            let key = evaluate(at("key")?, db)?.ok_or("undefined Map key")?;
            let entries = base.get("$map").and_then(Value::as_array).ok_or("Map operation on non-Map")?;
            let found = entries.iter().find(|item| same(&item[0], &key));
            if op == "has" {
                Some(json!(found.is_some()))
            } else {
                found.map(|item| item[1].clone())
            }
        }
        _ => panic!("unsupported IR expression {op}"),
    })
}
fn same(a: &Value, b: &Value) -> bool {
    if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
        return a == b;
    }
    if let (Some(a), Some(b)) = (a.get("$map").and_then(Value::as_array), b.get("$map").and_then(Value::as_array)) {
        return a.len() == b.len() && a.iter().all(|x| b.iter().any(|y| same(&x[0], &y[0]) && same(&x[1], &y[1])));
    }
    match (a, b) {
        (Value::Array(a), Value::Array(b)) => a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same(a, b)),
        (Value::Object(a), Value::Object(b)) => a.len() == b.len() && a.iter().all(|(k, v)| b.get(k).is_some_and(|b| same(v, b))),
        _ => a == b,
    }
}
pub fn execute(actions: &[Value], db: &mut impl Adapter) -> Result<(), String> {
    for action in actions {
        match action["op"].as_str().ok_or("missing action")? {
            "reset" => db.reset(),
            "parse" => db.parse(action["source"].as_str().ok_or("missing source")?)?,
            "raise" => return Err(action["message"].as_str().ok_or("missing exception")?.into()),
            "throws" => {
                let error = execute(action["actions"].as_array().ok_or("throw actions")?, db).err();
                let negated = action["negated"].as_bool().ok_or("throw negation")?;
                if error.is_some() == negated {
                    return Err(format!("toThrow negated={negated}, actual={error:?}"));
                }
            }
            "assert" => {
                let a = evaluate(&action["actual"], db)?;
                let b = evaluate(&action["expected"], db)?;
                let matcher = action["matcher"].as_str().ok_or("missing matcher")?;
                let passed = match matcher {
                    "toBe" => {
                        if a.as_ref().is_some_and(|v| v.is_array() || v.is_object())
                            || b.as_ref().is_some_and(|v| v.is_array() || v.is_object())
                        {
                            panic!("Object identity requires a specific adapter");
                        }
                        match (&a, &b) {
                            (None, None) => true,
                            (Some(a), Some(b)) => same(a, b),
                            _ => false,
                        }
                    }
                    "toEqual" => match (&a, &b) {
                        (None, None) => true,
                        (Some(a), Some(b)) => same(a, b),
                        _ => false,
                    },
                    "toBeUndefined" => a.is_none(),
                    "toBeDefined" => a.is_some(),
                    "toHaveLength" => {
                        let length = a.as_ref().and_then(|v| v.as_array().map(Vec::len).or_else(|| v.as_str().map(crate::util::js::len16)));
                        length.map(|v| v as f64) == b.as_ref().and_then(Value::as_f64)
                    }
                    "toContain" => match (&a, &b) {
                        (Some(Value::Array(a)), Some(b)) => a.iter().any(|v| same(v, b)),
                        (Some(Value::String(a)), Some(Value::String(b))) => a.contains(b),
                        _ => return Err("toContain on unsupported values".into()),
                    },
                    "toBeGreaterThan" | "toBeLessThan" | "toBeGreaterThanOrEqual" | "toBeLessThanOrEqual" => {
                        let a = a.as_ref().and_then(Value::as_f64).ok_or("comparison actual not numeric")?;
                        let b = b.as_ref().and_then(Value::as_f64).ok_or("comparison expected not numeric")?;
                        match matcher {
                            "toBeGreaterThan" => a > b,
                            "toBeLessThan" => a < b,
                            "toBeGreaterThanOrEqual" => a >= b,
                            _ => a <= b,
                        }
                    }
                    _ => panic!("unsupported matcher {matcher}"),
                };
                let negated = action["negated"].as_bool().ok_or("missing assertion negation")?;
                if passed == negated {
                    return Err(format!("{matcher}, negated={negated}: actual {a:?}, expected {b:?}"));
                }
            }
            other => panic!("unsupported IR action {other}"),
        }
    }
    Ok(())
}

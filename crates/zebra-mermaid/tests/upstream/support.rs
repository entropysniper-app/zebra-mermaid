//! Shared runtime for mechanically translated parser assertions, compiled inside the library tests.
//! Expected values and assertion operations come from unchanged upstream test callbacks.
use serde_json::{json, Value};
pub trait Adapter {
    fn reset(&mut self);
    fn parse(&mut self, source: &str) -> Result<(), String>;
    fn root(&self, key: &str) -> Result<Option<Value>, String>;
    fn invoke(&mut self, _action: &Value) -> Result<(), String> {
        panic!("unsupported model operation")
    }
    fn observe(&mut self, _method: &str, _enabled: bool) {
        panic!("unsupported observation")
    }
}
pub fn map(entries: impl IntoIterator<Item = (String, Value)>) -> Value {
    json!({"$map":entries.into_iter().map(|(k,v)|json!([k,v])).collect::<Vec<_>>()})
}
fn evaluate(e: &Value, db: &impl Adapter) -> Result<Option<Value>, String> {
    evaluate_scoped(e, db, None)
}
fn evaluate_scoped(e: &Value, db: &impl Adapter, scope_item: Option<&Value>) -> Result<Option<Value>, String> {
    let at = |key: &str| e.get(key).ok_or_else(|| format!("missing IR field {key}"));
    let op = at("op")?.as_str().ok_or("IR op is not a string")?;
    Ok(match op {
        "undefined" => None,
        "item" => Some(scope_item.expect("predicate item").clone()),
        "equal" | "notEqual" => {
            let a = evaluate_scoped(at("left")?, db, scope_item)?;
            let b = evaluate_scoped(at("right")?, db, scope_item)?;
            assert!(
                !a.as_ref().is_some_and(|v| v.is_object() || v.is_array()) && !b.as_ref().is_some_and(|v| v.is_object() || v.is_array()),
                "object identity requires explicit projection"
            );
            let equal = match (a, b) {
                (None, None) => true,
                (Some(a), Some(b)) => same_key(&a, &b),
                _ => false,
            };
            Some(json!(if op == "notEqual" { !equal } else { equal }))
        }
        "and" | "or" => {
            let left = evaluate_scoped(at("left")?, db, scope_item)?;
            if op == "and" && !truthy(left.as_ref()) || op == "or" && truthy(left.as_ref()) {
                left
            } else {
                evaluate_scoped(at("right")?, db, scope_item)?
            }
        }
        "keys" => {
            let base = evaluate_scoped(at("base")?, db, scope_item)?.expect("keys base");
            Some(Value::Array(base.get("$map").and_then(Value::as_array).expect("Map keys").iter().map(|v| v[0].clone()).collect()))
        }
        "mapOwnKeys" => {
            let base = evaluate_scoped(at("base")?, db, scope_item)?.expect("Object.keys input");
            assert!(base.get("$map").is_some(), "Object.keys on an unprojected object");
            // A native JavaScript Map has no enumerable own string properties.
            Some(json!([]))
        }
        "arrayFrom" => {
            let base = evaluate_scoped(at("base")?, db, scope_item)?.expect("iterable base");
            assert!(base.is_array(), "unsupported iterable");
            Some(base)
        }
        "find" | "filter" => {
            let base = evaluate_scoped(at("base")?, db, scope_item)?.ok_or("find on undefined")?;
            let mut found = vec![];
            for v in base.as_array().expect("find array") {
                if truthy(evaluate_scoped(at("predicate")?, db, Some(v))?.as_ref()) {
                    found.push(v.clone());
                    if op == "find" {
                        break;
                    }
                }
            }
            if op == "find" {
                found.into_iter().next()
            } else {
                Some(Value::Array(found))
            }
        }
        "literal" => Some(at("value")?.clone()),
        "root" => db.root(at("key")?.as_str().ok_or("root key")?).expect("unsupported adapter root"),
        "cleanupComments" => {
            let source = evaluate_scoped(at("source")?, db, scope_item)?.expect("cleanup input");
            Some(json!(crate::preprocess::cleanup_comments(source.as_str().expect("cleanup string")).0))
        }
        "array" => Some(Value::Array(
            at("items")?
                .as_array()
                .ok_or("IR items")?
                .iter()
                .map(|v| evaluate_scoped(v, db, scope_item)?.ok_or_else(|| "undefined array entry requires explicit adapter".into()))
                .collect::<Result<_, String>>()?,
        )),
        "object" => {
            let mut result = serde_json::Map::new();
            for item in at("entries")?.as_array().ok_or("IR entries")? {
                let key = item[0].as_str().ok_or("IR object key")?;
                let value = evaluate_scoped(&item[1], db, scope_item)?.ok_or("undefined object entry requires explicit adapter")?;
                result.insert(key.into(), value);
            }
            Some(Value::Object(result))
        }
        "map" => {
            let mut result = vec![];
            for item in at("entries")?.as_array().ok_or("IR entries")? {
                let key = evaluate_scoped(&item[0], db, scope_item)?.ok_or("undefined map key")?;
                let val = evaluate_scoped(&item[1], db, scope_item)?.ok_or("undefined map value")?;
                result.push(json!([key, val]));
            }
            Some(json!({"$map":result}))
        }
        "property" => {
            let base = evaluate_scoped(at("base")?, db, scope_item)?.ok_or("Cannot read properties of undefined")?;
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
            let base = evaluate_scoped(at("base")?, db, scope_item)?.ok_or("Map operation on undefined")?;
            let key = evaluate_scoped(at("key")?, db, scope_item)?.ok_or("undefined Map key")?;
            let entries = base.get("$map").and_then(Value::as_array).ok_or("Map operation on non-Map")?;
            let found = entries.iter().find(|item| same_key(&item[0], &key));
            if op == "has" {
                Some(json!(found.is_some()))
            } else {
                found.map(|item| item[1].clone())
            }
        }
        _ => panic!("unsupported IR expression {op}"),
    })
}
fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|n| n != 0.0 && !n.is_nan()),
        Some(Value::String(s)) => !s.is_empty(),
        Some(_) => true,
    }
}
fn partial(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Object(a), Value::Object(b)) => b.iter().all(|(k, v)| a.get(k).is_some_and(|a| partial(a, v))),
        (Value::Array(a), Value::Array(b)) => a.len() == b.len() && a.iter().zip(b).all(|(a, b)| partial(a, b)),
        _ => same(actual, expected),
    }
}
fn same_key(a: &Value, b: &Value) -> bool {
    if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
        a == b
    } else {
        same(a, b)
    }
}
fn same(a: &Value, b: &Value) -> bool {
    if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
        return a == b && (a != 0.0 || a.is_sign_negative() == b.is_sign_negative());
    }
    if let (Some(a), Some(b)) = (a.get("$map").and_then(Value::as_array), b.get("$map").and_then(Value::as_array)) {
        return a.len() == b.len() && a.iter().all(|x| b.iter().any(|y| same_key(&x[0], &y[0]) && same(&x[1], &y[1])));
    }
    match (a, b) {
        (Value::Array(a), Value::Array(b)) => a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same(a, b)),
        (Value::Object(a), Value::Object(b)) => a.len() == b.len() && a.iter().all(|(k, v)| b.get(k).is_some_and(|b| same(v, b))),
        _ => a == b,
    }
}
fn error_matches(error: &str, pattern: &Value) -> bool {
    assert_eq!(pattern["flags"], "", "unsupported regex flags");
    // Exact language of this original regex, including ECMAScript whitespace. Additional
    // patterns must get their own translation, never be treated as a generic throw.
    assert_eq!(pattern["source"], "(Expecting\\s'STR'|Unrecognized\\stext)", "unsupported error regex");
    [("Expecting", "'STR'"), ("Unrecognized", "text")].into_iter().any(|(before, after)| {
        error.match_indices(before).any(|(at, _)| {
            let rest = &error[at + before.len()..];
            rest.chars().next().is_some_and(|c| crate::util::chars::is_space(c) && rest[c.len_utf8()..].starts_with(after))
        })
    })
}
pub fn execute(actions: &[Value], db: &mut impl Adapter) -> Result<(), String> {
    execute_scoped(actions, db, None)
}
fn execute_scoped(actions: &[Value], db: &mut impl Adapter, scope_item: Option<&Value>) -> Result<(), String> {
    for action in actions {
        match action["op"].as_str().ok_or("missing action")? {
            "forEach" => {
                let array = evaluate_scoped(&action["base"], db, scope_item)?.expect("forEach base");
                for item in array.as_array().expect("forEach array") {
                    execute_scoped(action["actions"].as_array().expect("forEach body"), db, Some(item))?;
                }
            }
            "reset" => db.reset(),
            "invoke" => db.invoke(action)?,
            "observe" => {
                db.observe(action["method"].as_str().expect("observed method"), action["enabled"].as_bool().expect("observation mode"))
            }
            "callAssert" => {
                let calls = db.root(action["method"].as_str().expect("call root")).expect("call adapter").expect("call list");
                let expected = action["args"].as_array().expect("call arguments");
                let passed = calls.as_array().expect("calls array").iter().any(|call| {
                    let call = call.as_array().expect("call args");
                    call.len() == expected.len()
                        && call.iter().zip(expected).all(|(a, b)| {
                            if b["op"] == "stringContaining" {
                                a.as_str().is_some_and(|s| s.contains(b["value"].as_str().expect("substring")))
                            } else {
                                evaluate(b, db).expect("call expectation").is_some_and(|b| same(a, &b))
                            }
                        })
                });
                if passed == action["negated"].as_bool().expect("call negation") {
                    return Err(format!("call assertion differs: {calls:?} vs {expected:?}"));
                }
            }
            "parse" => {
                if let Some(expr) = action.get("sourceExpr") {
                    let source = evaluate(expr, db)?.expect("parse input");
                    db.parse(source.as_str().expect("parse input string"))?;
                } else {
                    db.parse(action["source"].as_str().expect("missing source"))?;
                }
            }
            "raise" => return Err(action["message"].as_str().ok_or("missing exception")?.into()),
            "throws" => {
                let error = execute_scoped(action["actions"].as_array().ok_or("throw actions")?, db, scope_item).err();
                let negated = action["negated"].as_bool().ok_or("throw negation")?;
                let matches = error.as_ref().is_some_and(|e| {
                    action.get("pattern").is_none_or(|p| error_matches(e, p))
                        && action.get("message").is_none_or(|m| e.contains(m.as_str().expect("throw message")))
                });
                if matches == negated {
                    return Err(format!("toThrow negated={negated}, actual={error:?}"));
                }
            }
            "assert" => {
                let a = evaluate_scoped(&action["actual"], db, scope_item)?;
                let b = evaluate_scoped(&action["expected"], db, scope_item)?;
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
                    "toMatchObject" => a.as_ref().zip(b.as_ref()).is_some_and(|(a, b)| partial(a, b)),
                    "toBeUndefined" => a.is_none(),
                    "toBeDefined" => a.is_some(),
                    "toBeTruthy" | "toBeFalsy" => {
                        let value = match &a {
                            None | Some(Value::Null) => false,
                            Some(Value::Bool(b)) => *b,
                            Some(Value::Number(n)) => n.as_f64().is_some_and(|n| n != 0.0 && !n.is_nan()),
                            Some(Value::String(s)) => !s.is_empty(),
                            Some(_) => true,
                        };
                        if matcher == "toBeTruthy" {
                            value
                        } else {
                            !value
                        }
                    }
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

/// Preserve a Vitest callback's time limit, including tests whose purpose is to detect a hang.
/// Panics remain test failures; they are never converted into parser errors by a catch action.
#[cfg(feature = "sequence")]
pub fn with_timeout(timeout_ms: u64, run: impl FnOnce() + Send + 'static) {
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(run));
        let _ = send.send(result);
    });
    match receive.recv_timeout(std::time::Duration::from_millis(timeout_ms)) {
        Ok(Ok(())) => (),
        Ok(Err(panic)) => std::panic::resume_unwind(panic),
        Err(error) => panic!("Original upstream timeout {timeout_ms} ms: {error}"),
    }
}

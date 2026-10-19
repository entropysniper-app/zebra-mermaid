//! Exact assertion interpreter for the frozen Gantt date tests. Dates retain their type;
//! NaN and undefined use explicit tags instead of being silently serialized to null.
use super::{
    axis, dates,
    js_date::{JsDate, Zone},
};
use serde_json::{json, Value};
use std::sync::LazyLock;
static DATA: LazyLock<Value> =
    LazyLock::new(|| serde_json::from_str(include_str!("../../../../tests/upstream/ir/gantt-dates.json")).unwrap());
fn number(n: f64) -> Value {
    if n.is_nan() {
        json!({"$number":"NaN"})
    } else {
        json!(n)
    }
}
fn numeric(v: &Value) -> f64 {
    if v.get("$number") == Some(&json!("NaN")) {
        f64::NAN
    } else {
        v.as_f64().expect("numeric value")
    }
}
fn undefined() -> Value {
    json!({"$undefined":true})
}
fn date(d: JsDate) -> Value {
    json!({"$date":number(d.time())})
}
fn as_date(v: &Value) -> JsDate {
    Zone::UTC.at(numeric(v.get("$date").expect("typed Date")))
}
fn unit(s: &str) -> dates::Unit {
    match s {
        "ms" => dates::Unit::Millisecond,
        "s" => dates::Unit::Second,
        "m" => dates::Unit::Minute,
        "h" => dates::Unit::Hour,
        "d" => dates::Unit::Day,
        "w" => dates::Unit::Week,
        "M" => dates::Unit::Month,
        "y" => dates::Unit::Year,
        _ => panic!("unsupported unit {s}"),
    }
}
fn unit_text(unit: dates::Unit) -> &'static str {
    match unit {
        dates::Unit::Millisecond => "ms",
        dates::Unit::Second => "s",
        dates::Unit::Minute => "m",
        dates::Unit::Hour => "h",
        dates::Unit::Day => "d",
        dates::Unit::Week => "w",
        dates::Unit::Month => "M",
        dates::Unit::Year => "y",
    }
}
fn evaluate(e: &Value) -> Value {
    let op = e["op"].as_str().expect("expression op");
    match op {
        "undefined" => undefined(),
        "literal" => e["value"].clone(),
        "number" => {
            assert_eq!(e["value"], "NaN");
            number(f64::NAN)
        }
        "date" => date(Zone::UTC.at(numeric(&evaluate(&e["time"])))),
        "array" => Value::Array(e["items"].as_array().unwrap().iter().map(evaluate).collect()),
        "length" => json!(evaluate(&e["base"]).as_array().expect("array length").len()),
        "getTime" => number(as_date(&evaluate(&e["base"])).time()),
        "isNaN" => json!(numeric(&evaluate(&e["base"])).is_nan()),
        _ => {
            let args: Vec<_> = e["args"].as_array().expect("call arguments").iter().map(evaluate).collect();
            let s = |i: usize| args[i].as_str().expect("string argument");
            let n = |i: usize| numeric(&args[i]);
            match op {
                "parseStrict" => {
                    dates::parse_strict(s(0), s(1), dates::compile(s(1)).as_deref(), &as_date(&args[2])).map_or_else(undefined, date)
                }
                "format" => dates::format(&as_date(&args[0]), s(1)).map_or_else(undefined, Value::String),
                "add" => date(dates::add(&as_date(&args[0]), n(1), unit(s(2)))),
                "parseDuration" => {
                    let (n, u) = dates::parse_duration(s(0)).unwrap_or((f64::NAN, dates::Unit::Millisecond));
                    json!([number(n), unit_text(u)])
                }
                "timeFormat" => Value::String(axis::time_format(s(0), &as_date(&args[1]))),
                "autoTicks" => Value::Array(axis::auto_ticks(n(0), n(1), n(2), Zone::UTC).into_iter().map(number).collect()),
                "intervalTicks" => axis::interval_ticks(s(0), s(1), n(2), n(3), n(4), Zone::UTC)
                    .map_or_else(undefined, |xs| Value::Array(xs.into_iter().map(number).collect())),
                _ => panic!("unsupported date expression {op}"),
            }
        }
    }
}
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(_), Value::Number(_)) => a.as_f64() == b.as_f64(),
        (Value::Array(a), Value::Array(b)) => a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same(a, b)),
        (Value::Object(a), Value::Object(b)) => a.len() == b.len() && a.iter().all(|(k, v)| b.get(k).is_some_and(|b| same(v, b))),
        _ => a == b,
    }
}
fn check(id: &str) {
    let r = DATA["records"].as_array().unwrap().iter().find(|r| r["id"] == id).unwrap();
    for a in r["assertions"].as_array().unwrap() {
        let actual = evaluate(&a["actual"]);
        let expected = evaluate(&a["expected"]);
        let passed = match a["matcher"].as_str().unwrap() {
            "toBe" | "toEqual" => same(&actual, &expected),
            "toBeUndefined" => actual == undefined(),
            "toBeLessThanOrEqual" => numeric(&actual) <= numeric(&expected),
            other => panic!("unsupported date matcher {other}"),
        };
        assert!(passed, "{}: {} actual={actual:?} expected={expected:?}", r["name"], a["matcher"]);
    }
}
include!("gantt_dates_generated.rs");
#[test]
fn adds_exact_time_units() {
    let start = Zone::UTC.local(2013.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(dates::add(&start, 2.0, dates::Unit::Hour).time() - start.time(), 7200000.0);
    assert_eq!(dates::add(&start, 2.0, dates::Unit::Minute).time() - start.time(), 120000.0);
    assert_eq!(dates::add(&start, 0.005, dates::Unit::Second).time() - start.time(), 5.0);
    assert_eq!(dates::add(&start, 20.0, dates::Unit::Millisecond).time() - start.time(), 20.0);
}

use serde_json::Value;
use std::{env,fs};
use zebra_mermaid::{config::{ConfigValue,json::parse_json},preprocess::preprocess};
fn main(){
 let path=env::args().nth(1).unwrap();let bytes=fs::read(path).unwrap();let data:Value=serde_json::from_slice(&bytes).unwrap();
 let mut successes=0;let mut errors=0;
 for (i,c) in data["cases"].as_array().unwrap().iter().enumerate(){let e=&c["expected"];
  match preprocess(c["source"].as_str().unwrap()){
   Ok(p)=>{assert_eq!(e["ok"],true,"case {i}");assert_eq!(p.text,e["text"],"case {i}");assert_eq!(p.with_comments,e["withComments"],"case {i}");assert_eq!(serde_json::json!(p.title),e["title"],"case {i}");assert_eq!(ConfigValue::Object(p.config),parse_json(&e["config"].to_string()).unwrap(),"case {i}");assert_eq!(p.line_offset,e["lineOffset"].as_u64().unwrap() as usize,"case {i}");successes+=1;},
   Err(p)=>{assert_eq!(e["ok"],false,"case {i}");assert_eq!(p.code.as_str(),e["code"].as_str().unwrap());assert_eq!(p.message,e["message"].as_str().unwrap());errors+=1;}
  }
 }
 println!("{}",serde_json::json!({"cases":successes+errors,"successes":successes,"failures":errors,"scope":"Frozen committed public preprocessing API against only independently generated Node views/config/metadata/errors. No latency statistics."}));
}

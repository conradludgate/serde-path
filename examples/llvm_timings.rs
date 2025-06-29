use std::io::{Read, stdin, stdout};

use serde_json::Serializer;
use serde_path::{
    FilterChain, hlist, json_ser::JsonSer, map::Map, predicate::NotEq, select::Select,
};

fn main() {
    // let mut json = vec![];
    // stdin().read_to_end(&mut json).unwrap();

    // equivalent to `.traceEvents | map(select(.ph != "X"))` in jq.
    let path = hlist![
        "traceEvents",
        Map(Select(NotEq::new("ph", "X".to_string())))
    ];

    let mut de = serde_json::Deserializer::from_reader(stdin());
    path.filter(JsonSer(Serializer::pretty(stdout())), &mut de)
        .unwrap();
    de.end().unwrap();
}

#[test]
fn test() {
    use std::io::Cursor;

    use serde_json::json;

    let stdin = Cursor::new(
        json!({
            "traceEvents": [
                {"ph": "M"}
            ]
        })
        .to_string()
        .into_bytes(),
    );

    // equivalent to `.traceEvents | map(select(.ph != "X"))` in jq.
    let path = hlist![
        "traceEvents",
        Map(Select(NotEq::new("ph", "X".to_string())))
    ];

    let mut de = serde_json::Deserializer::from_reader(stdin);
    path.filter(JsonSer(Serializer::pretty(stdout())), &mut de)
        .unwrap();
    de.end().unwrap();
}

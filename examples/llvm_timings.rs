use std::io::{Read, stdin, stdout};

use serde_json::Serializer;
use serde_path::{FilterChain, hlist, json_ser::JsonSer, map_select::MapSelect, predicate::NotEq};

fn main() {
    let mut json = vec![];
    stdin().read_to_end(&mut json).unwrap();

    // equivalent to `.traceEvents | map(select(.ph != "X"))` in jq.
    let path = hlist!["traceEvents", MapSelect(NotEq::new("ph", "X"))];

    let mut de = serde_json::Deserializer::from_slice(&json);
    path.filter(JsonSer(Serializer::pretty(stdout())), &mut de)
        .unwrap();
    de.end().unwrap();
}

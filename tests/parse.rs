use pretty_assertions::assert_eq;
use std::collections::HashMap;

#[test] fn parse_junk0() {
  let (args,argv) = argmap::parse([
    "--long","5",
    "-x","6",
    "-n3",
    "hello",
    "-xvf","whatever.tgz",
    "-y=cool",
    "-x7",
    "world",
    "--z=13",
    "-z","12",
    "--",
    "hmm"
  ].iter());
  assert_eq![args, vec!["hello","world","hmm"]];
  assert_eq![argv, hash([
    ("long",vec!["5"]),
    ("x",vec!["6","7"]),
    ("n",vec!["3"]),
    ("v",vec![]),
    ("f",vec!["whatever.tgz"]),
    ("y",vec!["cool"]),
    ("z",vec!["13","12"]),
  ].iter())];
}

#[test] fn parse_junk1() {
  let (args,argv) = argmap::parse([
    "--hey=what",
    "-x", "5",
    "-x", "6",
    "hi",
    "-zn9",
    "-j", "3",
    "-i", "q",
    "-5",
    "--n", "-1312",
    "-xvf", "payload.tgz",
    "-j=zzz",
    "-",
    "whatever",
    "-w3",
    "--",
    "-cool",
    "--yes=xyz"
  ].iter());
  assert_eq![args, vec!["hi","-","whatever","-cool","--yes=xyz"]];
  assert_eq![argv, hash([
    ("hey",vec!["what"]),
    ("x",vec!["5","6"]),
    ("z",vec![]),
    ("j",vec!["3","zzz"]),
    ("i",vec!["q"]),
    ("5",vec![]),
    ("n",vec!["9","-1312"]),
    ("v",vec![]),
    ("f",vec!["payload.tgz"]),
    ("w",vec!["3"]),
  ].iter())];
}

#[test] fn parse_empty() {
  let empty: Vec<String> = vec![];
  let (args,argv) = argmap::parse(empty.iter());
  assert_eq![args, empty];
  assert_eq![argv, hash([].iter())];
}

#[test] fn parse_one_long_bool() {
  let empty: Vec<String> = vec![];
  let (args,argv) = argmap::parse(["--one"].iter());
  assert_eq![args, empty];
  assert_eq![argv, hash([
    ("one",vec![]),
  ].iter())];
}

#[test] fn parse_one_short_bool() {
  let empty: Vec<String> = vec![];
  let (args,argv) = argmap::parse(["-z"].iter());
  assert_eq![args, empty];
  assert_eq![argv, hash([
    ("z",vec![]),
  ].iter())];
}

#[test] fn parse_bool_at_dashdash() {
  let empty: Vec<String> = vec![];
  let (args,argv) = argmap::parse(["--q","--"].iter());
  assert_eq![args, empty];
  assert_eq![argv, hash([
    ("q",vec![]),
  ].iter())];
}

#[test] fn parse_negative_number_value() {
  let empty: Vec<String> = vec![];
  let (args,argv) = argmap::parse(["--n","-555"].iter());
  assert_eq![args, empty];
  assert_eq![argv, hash([
    ("n",vec!["-555"]),
  ].iter())];
}

#[test] fn parse_cluster_number() {
  let empty: Vec<String> = vec![];
  let (args,argv) = argmap::parse(["-abcdef123456"].iter());
  assert_eq![args, empty];
  assert_eq![argv, hash([
    ("a",vec![]),
    ("b",vec![]),
    ("c",vec![]),
    ("d",vec![]),
    ("e",vec![]),
    ("f",vec!["123456"]),
  ].iter())];
}

#[test] fn parse_single_boolean() {
  let (args,argv) = argmap::new().boolean("q").parse([
    "-x", "5",
    "-q", "1234",
    "--z=789",
  ].iter());
  assert_eq![args, vec!["1234"]];
  assert_eq![argv, hash([
    ("x",vec!["5"]),
    ("q",vec![]),
    ("z",vec!["789"]),
  ].iter())];
}

#[test] fn parse_boolean_nonalpha_break() {
  let empty: Vec<String> = vec![];
  let (args,argv) = argmap::new().boolean("q").parse([
    "-w-5", "-qrs@4"
  ].iter());
  assert_eq![args, empty];
  assert_eq![argv, hash([
    ("w",vec!["-5"]),
    ("q",vec![]),
    ("r",vec![]),
    ("s",vec!["@4"]),
  ].iter())];
}

#[test] fn parse_booleans_slice() {
  let (args,argv) = argmap::new().booleans(&["q","z"]).parse([
    "-q", "x", "-z", "y"
  ].iter());
  assert_eq![args, vec!["x","y"]];
  assert_eq![argv, hash([
    ("q",vec![]),
    ("z",vec![]),
  ].iter())];
}

#[test] fn parse_boolean_vec_ref() {
  let (args,argv) = argmap::new().booleans(&vec!["q","z"]).parse([
    "-q", "x", "-z", "y"
  ].iter());
  assert_eq![args, vec!["x","y"]];
  assert_eq![argv, hash([
    ("q",vec![]),
    ("z",vec![]),
  ].iter())];
}

fn hash<'a>(i: impl Iterator<Item=&'a (&'a str,Vec<&'a str>)>) -> HashMap<String,Vec<String>> {
  i.map(|(k,v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect())).collect()
}

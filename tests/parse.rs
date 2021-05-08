use pretty_assertions::assert_eq;
use std::collections::HashMap;

#[test] fn parse() {
  {
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

  {
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

  {
    let empty: Vec<String> = vec![];
    let (args,argv) = argmap::parse(empty.iter());
    assert_eq![args, empty];
    assert_eq![argv, hash([].iter())];
  }

  {
    let empty: Vec<String> = vec![];
    let (args,argv) = argmap::parse(["--one"].iter());
    assert_eq![args, empty];
    assert_eq![argv, hash([
      ("one",vec![]),
    ].iter())];
  }

  {
    let empty: Vec<String> = vec![];
    let (args,argv) = argmap::parse(["-z"].iter());
    assert_eq![args, empty];
    assert_eq![argv, hash([
      ("z",vec![]),
    ].iter())];
  }

  {
    let empty: Vec<String> = vec![];
    let (args,argv) = argmap::parse(["--q","--"].iter());
    assert_eq![args, empty];
    assert_eq![argv, hash([
      ("q",vec![]),
    ].iter())];
  }

  {
    let empty: Vec<String> = vec![];
    let (args,argv) = argmap::parse(["--n","-555"].iter());
    assert_eq![args, empty];
    assert_eq![argv, hash([
      ("n",vec!["-555"]),
    ].iter())];
  }

  {
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

  {
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
}

fn hash<'a>(i: impl Iterator<Item=&'a (&'a str,Vec<&'a str>)>) -> HashMap<String,Vec<String>> {
  i.map(|(k,v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect())).collect()
}

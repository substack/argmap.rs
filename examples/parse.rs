fn main() {
  let (args,argv) = argmap::parse(std::env::args());
  eprintln!["args={:?}", &args];
  eprintln!["argv={:?}", &argv];
}

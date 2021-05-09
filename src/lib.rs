//! # argmap
//!
//! parse command-line arguments into a hashmap and vec of positional args
//!
//! This library does not make you write a struct or format a help message. Everything stays a `String`
//! and it's up to you to do whatever kind of parsing you need.
//!
//! You provide an iterator of items that implement ToString and you get back a 2-tuple of `(args,argv)`:
//!
//! ``` rust
//! let (args,argv) = argmap::parse(std::env::args());
//! eprintln!["args={:?}", &args]; // args is a Vec<String> of positional arguments
//! eprintln!["argv={:?}", &argv]; // argv is a HashMap<String,Vec<String>>
//! ```
//!
//! Long (`--file`) and short (`-x`) options, with or without equal signs, clustered short options
//! (example: `tar -xvf file.tgz`) and non-alpha short-circuiting (example: `tail -n1`) are all supported.
//! You can also have numeric flags but not in short clusters.
//!
//! Here's an example of the junk you can throw at this parser:
//!
//! ``` sh
//! $ cargo run -q --example parse -- -z 5 -y=6 -y8 --msg cool -7 --here=there \
//!   -xvf file.tgz -qrs=1234 -n -555 one two three -abc+5 -c-6 -- four -z 0
//! args=["target/debug/examples/parse", "one", "two", "three", "four", "-z", "0"]
//! argv={"z": ["5"], "7": [], "y": ["6", "8"], "x": [], "v": [], "f": ["file.tgz"], "a": [], "b": [], "here": ["there"], "n": ["-555"], "qrs": ["1234"], "c": ["+5", "-6"], "msg": ["cool"]}
//! ```
//!
//! The values for the `argv` HashMap are `Vec<String>` instead of `String` because you may have the
//! same option specified multiple times. If you only want to deal with a single value for a given key,
//! you can use the `.first()` or `.last()` inside an `.and_then()`:
//!
//! ``` rust
//! let (args,argv) = argmap::parse(std::env::args());
//! let cool = argv.get("cool").and_then(|v| v.last());
//! ```
//!
//! `HashMap` has more ergonomic field access than any argument parser could hope to create and you can
//! use the knowledge you already have for how to work with it instead of learning an argument-parser
//! specific api.
//!
//! Likewise, many of the usual features that a command-line parser has (aliasing and default values for
//! example) can be obtained from methods on the core `Option` type such as `.or_else()`, `.and_then()`,
//! or `.unwrap_or()`.
//!
//! Here is a longer example because how to string all of those together in a useful way is not
//! necessarily obvious. This example is a word count program like `wc`, but overly-simplified and
//! somewhat inaccurate for the sake of brevity.
//!
//! ``` rust
//! use std::{io,fs::File};
//!
//! type Error = Box<dyn std::error::Error+Send+Sync>;
//! type R = Box<dyn io::Read+Unpin>;
//!
//! fn main() -> Result<(),Error> {
//!   let (args,argv) = argmap::new()
//!     .boolean("h").boolean("help")
//!     .boolean("c").boolean("bytes")
//!     .boolean("w").boolean("words")
//!     .boolean("l").boolean("lines")
//!     .parse(std::env::args());
//!   if argv.contains_key("h") || argv.contains_key("help") {
//!     indoc::printdoc![r#"usage: {} {{OPTIONS}} [FILE]
//!
//!       Count the number of bytes, words, or lines in a file or stdin.
//!
//!         -i, --infile  Count words from FILE or '-' for stdin (default).
//!         -c, --bytes   Show number of bytes.
//!         -w, --words   Show number of words.
//!         -l, --lines   Show number of lines.
//!         -h, --help    Show this message.
//!
//!     "#, args.get(0).unwrap_or(&"???".to_string())];
//!     return Ok(());
//!   }
//!
//!   let mut show_bytes = argv.contains_key("c") || argv.contains_key("bytes");
//!   let mut show_words = argv.contains_key("w") || argv.contains_key("words");
//!   let mut show_lines = argv.contains_key("l") || argv.contains_key("lines");
//!   if !show_bytes && !show_words && !show_lines {
//!     show_bytes = true;
//!     show_words = true;
//!     show_lines = true;
//!   }
//!
//!   let stdin_file = "-".to_string();
//!   let infile = argv.get("infile").and_then(|v| v.first()) // --infile=file
//!     .or_else(|| argv.get("i").and_then(|v| v.first())) // -i file
//!     .or_else(|| args.get(1)) // first positional arg after $0
//!     .unwrap_or(&stdin_file) // default value: "-"
//!     .as_str();
//!
//!   let mut stream: R = match infile {
//!     "-" => Box::new(io::stdin()),
//!     f => Box::new(File::open(f)?),
//!   };
//!   let mut buf = vec![0;4096];
//!   let mut byte_count = 0;
//!   let mut word_count = 0;
//!   let mut line_count = 0;
//!   loop {
//!     let len = stream.read(&mut buf)?;
//!     if len == 0 { break }
//!     byte_count += len;
//!     let s = std::str::from_utf8(&buf[0..len])?;
//!     word_count += s.split_whitespace().count();
//!     line_count += s.lines().count();
//!   }
//!   let mut outline = "".to_string();
//!   if show_lines { outline += &format!["{:>4} ", line_count] }
//!   if show_words { outline += &format!["{:>4} ", word_count] }
//!   if show_bytes { outline += &format!["{:>4} ", byte_count] }
//!   println!["{}", outline.trim_end()];
//!   Ok(())
//! }
//! ```
//!
//! This example also demonstrates the `.boolean()` method to tell the parser that certain fields are to
//! be interpreted as boolean values. Right now that is the only configuration available.
//!
//! Many libraries that do parsing also provide help messages, but I much prefer to write them out by
//! hand as in the example above. This way, I have more control over how the help info is presented and
//! formatted to be maximally helpful. For example, some flags might only make sense in combination with
//! certain other flags, but that is hard to show with formatting options presented by an automated
//! tool. And if the help message gets too long you can always split it out into a separate file.

use std::collections::{HashMap,HashSet};

pub struct ArgMap {
  pub boolean: HashSet<String>,
}

pub type Map = HashMap<String,Vec<String>>;
pub type List = Vec<String>;

impl ArgMap {
  /// Create a new ArgMap instance.
  pub fn new() -> Self {
    Self {
      boolean: HashSet::new(),
    }
  }
  /// Set a key to be treated as a boolean argument, where an argument that follows a boolean
  /// argument will not be treated as the key's value.
  pub fn boolean<T>(mut self, key: T) -> Self where T: ToString {
    self.boolean.insert(key.to_string());
    self
  }
  /// Parse an iterator of string arguments into a 2-tuple of positional arguments and a
  /// HashMap mapping String keys to Vec<String> values.
  pub fn parse<T>(&mut self, input: impl Iterator<Item=T>) -> (List,Map) where T: ToString {
    let mut args: List = vec![];
    let mut argv: Map = HashMap::new();
    let mut key: Option<String> = None;
    let mut dashdash = false;
    for x in input {
      let s = x.to_string();
      if dashdash {
        args.push(s);
        continue;
      }
      if s == "--" {
        dashdash = true;
      } else if s == "-" {
        args.push(s);
      } else if s.starts_with("--") {
        if let Some(k) = &key {
          argv.insert(k.clone(), vec![]);
          key = None;
        }
        let k = s[2..].to_string();
        if let Some(i) = k.find("=") {
          set(&mut argv, &k[0..i].to_string(), &k[i+1..].to_string());
        } else if self.boolean.contains(&k) {
          set_bool(&mut argv, &k)
        } else {
          key = Some(k);
        }
      } else if s.starts_with("-") {
        if let Some(k) = &key {
          if is_num(&s[1..2]) {
            set(&mut argv, &k, &s.to_string());
            key = None;
            continue;
          }
          set_bool(&mut argv, &k);
          argv.insert(k.clone(), vec![]);
          key = None;
        }
        if let Some(i) = s.find("=") {
          let sk = s[1..i].to_string();
          let sv = s[i+1..].to_string();
          set(&mut argv, &sk, &sv);
        } else {
          let mut jump = false;
          for i in 1..s.len()-1 {
            let k = s[i..i+1].to_string();
            if let Some(sk) = &key {
              if is_num(&k) || short_break(&k) {
                set(&mut argv, sk, &s[i..].to_string());
                key = None;
                jump = true;
                break;
              } else {
                set_bool(&mut argv, &sk);
              }
              key = None;
            }
            if self.boolean.contains(&k) {
              set_bool(&mut argv, &k);
            } else {
              key = Some(k);
            }
          }
          if jump { continue }
          let k = s[s.len()-1..].to_string();
          if let Some(sk) = &key {
            if self.boolean.contains(&k) {
              set_bool(&mut argv, sk);
              set_bool(&mut argv, &k);
            } else if is_num(&k) || short_break(&k) {
              set(&mut argv, sk, &k);
              key = None;
            } else {
              set_bool(&mut argv, sk);
              key = Some(k);
            }
          } else if self.boolean.contains(&k) {
            set_bool(&mut argv, &k);
          } else {
            key = Some(k);
          }
        }
      } else if let Some(k) = key {
        set(&mut argv, &k, &s);
        key = None;
      } else {
        args.push(s);
      }
    }
    if let Some(k) = key {
      set_bool(&mut argv, &k);
    }
    (args,argv)
  }
}

/// Create a new ArgMap instance.
pub fn new() -> ArgMap {
  ArgMap::new()
}

/// Parse an iterator of string arguments into a 2-tuple of positional arguments and a
/// HashMap mapping String keys to Vec<String> values.
pub fn parse<T>(input: impl Iterator<Item=T>) -> (List,Map) where T: ToString {
  ArgMap::new().parse(input)
}

fn is_num(s: &str) -> bool {
  s.chars().nth(0).and_then(|c| Some('0' <= c && c <= '9')).unwrap_or(false)
}
fn short_break(s: &String) -> bool {
  s.chars().next()
    .and_then(|c| Some(!c.is_alphabetic()))
    .unwrap_or(false)
}

fn set(argv: &mut Map, key: &String, value: &String) {
  if let Some(values) = argv.get_mut(key) {
    values.push(value.clone());
  } else {
    argv.insert(key.clone(), vec![value.clone()]);
  }
}
fn set_bool(argv: &mut Map, key: &String) {
  if !argv.contains_key(key) {
    argv.insert(key.clone(), vec![]);
  }
}

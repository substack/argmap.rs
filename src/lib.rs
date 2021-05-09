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

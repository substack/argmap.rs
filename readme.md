# argmap

parse command-line arguments into a hashmap and vec of positional args

This library doesn't populate custom structs, format help messages, or convert types.

You provide an iterator of items that implement ToString and you get back a 2-tuple of
`(args,argv)` where:

* `args` is a `Vec<String>` of positional arguments
* `argv` is a `HashMap<String,Vec<String>>` of all the values that map to a `--key`

``` rust
let (args,argv) = argmap::parse(std::env::args());
eprintln!["args={:?}", &args];
eprintln!["argv={:?}", &argv];
```

Long (`--file`) and short (`-x`) options, with or without equal signs, clustered short options
(example: `tar -xvf file.tgz`) and non-alpha short-circuiting (example: `tail -n1`) are all supported.
You can also have numeric flags but not in short clusters.

Here's an example of the junk you can throw at this parser:

``` sh
$ cargo run -q --example parse -- -z 5 -y=6 -y8 --msg cool -7 --here=there \
  -xvf file.tgz -qrs=1234 -n -555 one two three -abc+5 -c-6 -- four -z 0
args=["target/debug/examples/parse", "one", "two", "three", "four", "-z", "0"]
argv={"z": ["5"], "7": [], "y": ["6", "8"], "x": [], "v": [], "f": ["file.tgz"], "a": [], "b": [], "here": ["there"], "n": ["-555"], "qrs": ["1234"], "c": ["+5", "-6"], "msg": ["cool"]}
```

The values for the `argv` HashMap are `Vec<String>` instead of `String` because you may have the
same option specified multiple times. If you only want to deal with a single value for a given key,
you can use the `.first()` or `.last()` inside an `.and_then()`:

``` rust
let (args,argv) = argmap::parse(std::env::args());
let cool = argv.get("cool").and_then(|v| v.last());
```

Boolean options will be stored as an empty `vec![]`. You can use `.contains_key()` to test for the
presence of a boolean flag:

``` rust
let (args,argv) = argmap::parse(std::env::args());
let show_help = argv.contains_key("h") || argv.contains_key("help");
```

`HashMap` has more ergonomic field access than any argument parser could hope to create and you can
use the knowledge you already have for how to work with it instead of learning an argument-parser
specific api.

Likewise, many of the usual features that a command-line parser has (aliasing and default values for
example) can be obtained from methods on the core `Option` type such as `.or_else()`, `.and_then()`,
or `.unwrap_or()`.

Here is a longer example because how to string all of those together in a useful way is not
necessarily obvious. This example is a word count program like `wc`, but overly-simplified and
somewhat inaccurate for the sake of brevity.

``` rust
use std::{io,fs::File};

type Error = Box<dyn std::error::Error+Send+Sync>;
type R = Box<dyn io::Read+Unpin>;

fn main() -> Result<(),Error> {
  let (args,argv) = argmap::new()
    .booleans(&[ "h", "help", "c", "bytes", "w", "words", "l", "lines" ])
    .parse(std::env::args());
  if argv.contains_key("h") || argv.contains_key("help") {
    indoc::printdoc![r#"usage: {} {{OPTIONS}} [FILE]

      Count the number of bytes, words, or lines in a file or stdin.

        -i, --infile  Count words from FILE or '-' for stdin (default).
        -c, --bytes   Show number of bytes.
        -w, --words   Show number of words.
        -l, --lines   Show number of lines.
        -h, --help    Show this message.

    "#, args.get(0).unwrap_or(&"???".to_string())];
    return Ok(());
  }

  let mut show_bytes = argv.contains_key("c") || argv.contains_key("bytes");
  let mut show_words = argv.contains_key("w") || argv.contains_key("words");
  let mut show_lines = argv.contains_key("l") || argv.contains_key("lines");
  if !show_bytes && !show_words && !show_lines {
    show_bytes = true;
    show_words = true;
    show_lines = true;
  }

  let stdin_file = "-".to_string();
  let infile = argv.get("infile").and_then(|v| v.first()) // --infile=file
    .or_else(|| argv.get("i").and_then(|v| v.first())) // -i file
    .or_else(|| args.get(1)) // first positional arg after $0
    .unwrap_or(&stdin_file) // default value: "-"
    .as_str();

  let mut stream: R = match infile {
    "-" => Box::new(io::stdin()),
    f => Box::new(File::open(f)?),
  };
  let mut buf = vec![0;4096];
  let mut byte_count = 0;
  let mut word_count = 0;
  let mut line_count = 0;
  loop {
    let len = stream.read(&mut buf)?;
    if len == 0 { break }
    byte_count += len;
    let s = std::str::from_utf8(&buf[0..len])?;
    word_count += s.split_whitespace().count();
    line_count += s.lines().count();
  }
  let mut outline = "".to_string();
  if show_lines { outline += &format!["{:>4} ", line_count] }
  if show_words { outline += &format!["{:>4} ", word_count] }
  if show_bytes { outline += &format!["{:>4} ", byte_count] }
  println!["{}", outline.trim_end()];
  Ok(())
}
```

This example also demonstrates how to tell the parser that certain fields are to be interpreted as
boolean values. Right now that is the only configuration available.

Many libraries that do parsing also provide help messages, but I much prefer to write them out by
hand as in the example above. This way, I have more control over how the help info is presented and
formatted to be maximally helpful. For example, some flags might only make sense in combination with
certain other flags, but that is hard to show with formatting options presented by an automated
tool. And if the help message gets too long you can always split it out into a separate file.

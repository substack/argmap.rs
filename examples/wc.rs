use std::{io,fs::File};

type Error = Box<dyn std::error::Error+Send+Sync>;
type R = Box<dyn io::Read+Unpin>;

fn main() -> Result<(),Error> {
  let (args,argv) = argmap::new()
    .boolean("h").boolean("help")
    .boolean("c").boolean("bytes")
    .boolean("w").boolean("words")
    .boolean("l").boolean("lines")
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

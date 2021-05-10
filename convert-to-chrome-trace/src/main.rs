use clap::Arg;
use serde::Serialize;
use std::{
  collections::BTreeMap,
  error::Error,
  fs::File,
  io::{BufRead, BufReader, BufWriter, Write},
  process::exit,
};
use trace_span::capnp;

#[derive(Clone, Debug, Serialize)]
struct TraceEvent<'a> {
  name: &'a str,
  cat: &'a str,
  ph: &'a str,
  ts: u64,
  pid: u64,
  tid: u64,
  args: &'a BTreeMap<&'a str, &'a str>,
}

fn process(input: &mut dyn BufRead, output: &mut dyn Write) -> Result<(), Box<dyn Error>> {
  write!(output, "[\n")?;

  let mut spans = vec![];

  while let Ok(deserialized) =
    capnp::serialize_packed::read_message(&mut *input, capnp::message::ReaderOptions::new())
  {
    let span = deserialized.get_root::<trace_span::Reader>().unwrap();

    let mut annotations = BTreeMap::new();
    let annot_reader = span.get_annotations()?;
    for item in annot_reader.iter() {
      let name = item.get_name()?;
      let value = item.get_value()?;

      annotations.insert(name, value);
    }

    let name = span.get_name()?;
    let cat = annotations.get("categories").map(|x| *x).unwrap_or("all");
    let pid = 0;
    let tid = annotations.get("tid").unwrap_or(&"0").parse().unwrap_or(0);

    let start = serde_json::to_string(&TraceEvent {
      name,
      cat,
      ph: "B",
      ts: span.get_start() / 1000,
      pid,
      tid,
      args: &annotations,
    })?;

    let end = serde_json::to_string(&TraceEvent {
      name,
      cat,
      ph: "E",
      ts: span.get_end() / 1000,
      pid,
      tid,
      args: &annotations,
    })?;

    spans.push((span.get_start(), start));
    spans.push((span.get_end(), end));
  }

  spans.sort_unstable_by_key(|(a, _)| *a);

  for (_, value) in spans {
    write!(output, "{},\n", value)?;
  }

  Ok(())
}

fn main() {
  let matches = clap::App::new("convert-to-chrome-trace")
    .about("Tool to convert caffeine traces into the chrome trace event format")
    .arg(
      Arg::with_name("input")
        .default_value("-")
        .help("Input file from which to read the trace log."),
    )
    .arg(
      Arg::with_name("output")
        .short("o")
        .long("output")
        .help("Output file to which to write the chrome trace log.")
        .default_value("-"),
    )
    .get_matches();

  let stdin_v;
  let stdout_v;

  let mut stdin;
  let mut stdout;
  let mut ifile;
  let mut ofile;

  let input: &mut dyn BufRead;
  let output: &mut dyn Write;

  let input_name = matches.value_of("input").unwrap();
  let output_name = matches.value_of("output").unwrap();

  if input_name != "-" {
    ifile = match File::open(input_name) {
      Ok(ifile) => BufReader::new(ifile),
      Err(e) => {
        eprintln!("Unable to open '{}': {}", input_name, e);
        exit(1);
      }
    };

    input = &mut ifile;
  } else {
    stdin_v = std::io::stdin();
    stdin = BufReader::new(stdin_v.lock());

    input = &mut stdin;
  }

  if output_name != "-" {
    ofile = match File::create(output_name) {
      Ok(ofile) => BufWriter::new(ofile),
      Err(e) => {
        eprintln!("Unable to open '{}': {}", output_name, e);
        exit(1);
      }
    };

    output = &mut ofile;
  } else {
    stdout_v = std::io::stdout();
    stdout = BufWriter::new(stdout_v.lock());

    output = &mut stdout;
  }

  if let Err(e) = process(input, output) {
    eprintln!("An error occurred while processing the log: {}", e);
    exit(1);
  }
}

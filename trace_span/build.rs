fn main() {
  capnpc::CompilerCommand::new()
    .src_prefix("src")
    .file("src/tracepoint.capnp")
    .run()
    .unwrap();
}

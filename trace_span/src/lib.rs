

pub extern crate capnp;

#[allow(dead_code)]
mod tracepoint_capnp {
  include!(concat!(env!("OUT_DIR"), "/tracepoint_capnp.rs"));
}

pub use tracepoint_capnp::trace_span::*;

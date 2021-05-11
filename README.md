# tracing-tools - Tools for working with caffeine trace files

This repo contains a set of tools for working with the trace files emitted by
[caffeine](https://github.com/insufficiently-caffeinated/caffeine).

## `convert-to-chrome-trace`
This binary converts a caffeine trace to one that can be viewed by chrome's
builtin trace viewer. To transform an existing trace file you can to
```sh
cargo run --bin convert-to-chrome-trace -- path/to/caffeine.trace --output trace.json
```
Then, you can open the trace in chrome by going to 
[about://tracing](about://tracing) and loading `trace.json` from there.

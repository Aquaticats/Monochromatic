# Build and execution

## Node scripts vs single-file executables

Direct `node <script>.ts` execution in `mise.toml` is the default for source-run
workspace tasks:

- Platform portability:
   Node runs consistently across the supported host set.
- Industry precedent:
   Oxlint and dprint use runtime platform detection.
- Performance:
   Startup cost is acceptable for portability.

Bun execution stays limited to explicit Bun islands that document why Node is not
the target runtime.

# monochromatic-lint-fuzz

Coverage-guided checks for the unified linter's native boundaries.
Targets cover JSONC configuration validation,
ordered merging,
the Rust sidecar test rule,
semantic explicit-type checking,
the implemented Markdown/MDX rules,
and the executable's per-source orchestration path.

The merge target reuses the repository's structured JSONC generator.
The configuration target combines raw syntax mutation with always-valid generated rule settings.
The Rust-style target combines arbitrary valid UTF-8 with bounded generated Rust syntax
whose test finding count is known independently of the rule.
Its controls distinguish inline test modules and `#[test]` functions from module declarations,
negated and key-value `cfg` attributes,
test-looking strings and comments,
and sidecar file names.
The semantic target initializes a fixed in-memory Rust crate,
submits generated/raw source only through the production session's source-overlay API,
and checks independent violation counts plus restoration after changed input.
Arbitrary bytes never enter the fixture metadata interpreter.
The Markdown target runs independently counted source fragments with LF,
CRLF,
and bare CR in both Markdown and MDX modes,
then checks arbitrary UTF-8.
It checks original-byte diagnostic/edit boundaries and reparses accepted fixed output.
The counted catalog now includes pipe-table conversion and semantic prose breaks.
The orchestration target calls `run_file::process_source`,
the call the executable makes for every file,
under a fixed configuration that selects virtual files by their paths.
Each input checks one hand-counted host with rustdoc,
doc tests,
quoted fences,
MDX,
or two Rust levels of nesting in LF or CRLF spelling,
projects an adversarial edit group from a virtual file to the host,
and then treats the raw input as a host in each language.
It asserts host-addressed findings,
no fix after a processing failure,
no emptied file,
a fixed point once the loop settles,
and a `core/fix-refused` finding on refusal.
`markdown/lfs-image-url` is absent from that configuration because it reads the filesystem.
Container campaigns run both generator controls and sidecar Clippy before instrumented compilation.
The existing nonempty-to-empty rewrite refusal remains an explicit allowed error.
The raw parser path may reject malformed input with a typed processing failure.
Fuzz-only process deadlines do not add a production parser timeout.

Generator unit tests verify that successful validation,
merging,
and real sidecar test checks are actually reached.

`mise run //package/linter/monochromatic-lint.fuzz:test:planted` proves the invariants can fail.
It copies the linter,
this package and their JSONC dependencies to this package's ignored `target/planted` directory,
plants one defect per fuzz target
(a rejected `warn` severity,
arrays replaced instead of concatenated by the merge,
a sidecar file name no longer exempt,
an initialized binding exempt from type annotations,
a trailing colon no longer heading punctuation,
and an unlabeled rustdoc fence no longer a doc test),
and requires a generator control to fail for each.
A plant that does not build counts as not noticed.
Results are retained under `target/verification/planted-*`.

The `fuzz_target!` input looks like a closure signature but is macro input grammar.
In libfuzzer-sys 0.4.13,
`src/lib.rs:247-295` expands the byte-input form into the named `__libfuzzer_sys_run` function.

Run campaigns through the bounded container tasks.
They tag their images with `MONOCHROMATIC_LINT_IMAGE_TAG`,
`development` by default,
so concurrent sessions can keep separate images.
Retain and replay minimized failures.
A clean campaign does not cover the file walker,
configuration lookup on disk,
atomic writes,
the LFS rule,
or the cli-git transaction implementation.

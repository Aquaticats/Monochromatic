# meow HCL evaluator, function library, formatter, and language server

Design appendix to `doc/planning/monorepo-manager-from-scratch-design.md`,
section "Open questions",
item "The HCL evaluator,
 function library,
 formatter,
 and language server,
given the gaps in `hcl-rs` and `hcl-edit`".

Deliberation only.
No meow code was written,
and nothing in the main worktree changed except the troubleshooting documents named in "Tool quirks documented"
and the vet reports named in "Vet reports".

Date of every measurement in this document:
2026-09-17.
Merged into "HCL tooling" in `doc/planning/monorepo-manager-from-scratch-design.md` on 2026-09-17.
Correction on the same day:
this research read `doc/decision/monorepo-manager-all-rust.md` as forbidding a C toolchain.
That record says the tool is written in Rust;
whether a dependency may compile C is an open user question,
first raised by the cache key hash vet.
The `tree-sitter-hcl` exit stands on accepting 5 invalid files.

Every decision it feeds applies to meow 0.x only (user,
 2026-09-17).

## How to read the evidence labels

Every factual claim carries one of two labels.

- Verified:
  followed by the probe,
   command,
   file,
   or line that produced it.
  Anyone can rerun the command and see the same output.
- Unverified:
  a judgement,
   a projection,
   or a statement about future meow code that does not exist yet.
  These are never used to disqualify an option;
  disqualification always rests on a Verified line.

## Environment and probe harness

Verified (`~/temp/agent/hcl-evaluator-2026-09-17/scripts/run-box.ts`,
and the per-run record in `~/temp/agent/hcl-evaluator-2026-09-17/data/logs/executions.jsonl`):
every build,
 benchmark,
 and third-party test suite ran through one runner that always executes

```sh
podman run --rm --init --memory=2g --cpus=2 --pids-limit=512 --ulimit nofile=4096:4096 \
  --security-opt label=disable --network=none \
  --volume "${HOME}/temp/agent/hcl-evaluator-2026-09-17:/w" \
  --volume "${HOME}/temp/agent/hcl-evaluator-target-2026-09-17:/t" \
  --volume "${HOME}/.rustup/toolchains/nightly-2026-09-12-x86_64-unknown-linux-gnu:/toolchain:ro" \
  --volume "${HOME}/.cargo/registry:/cargo-ro/registry:ro" \
  --env CARGO_HOME=/t/cargo-home --env CARGO_NET_OFFLINE=true --env CARGO_TERM_COLOR=never \
  --workdir <workdir> localhost/hashvet-rusttest:2 <command>
```

Verified (`executions.jsonl`,
 field `digest`):
the image is `localhost/hashvet-rusttest:2`,
digest `sha256:efe5b287e1875e44129878945b0a2cbc630bec17eec0b9b063fbf5e37822c3bf`,
with no network,
 no credentials,
 and no repository mount.
Verified:
the runner recorded 68 executions with identical bounds
(`memory=2g cpus=2 pids=512 nofile=4096`).

Verified:
Cargo target directories live under `~/temp/agent/hcl-evaluator-target-2026-09-17/`
and that directory contains `CACHEDIR.TAG`.

Clone revisions used for every source citation,
Verified by `git log --max-count=1`:

- `hashicorp/hcl` `4932c14`,
- `opentofu/opentofu` `0e041e5`,
- `zclconf/go-cty` `a918e11`,
- `martinohmann/hcl-rs` `f7b1759` (`hcl-rs` 0.19.8,
   `hcl-edit` 0.9.7,
   `hcl-primitives` 0.1.12),
- `opentofu/tofu-ls` `c37a827`,
- `hashicorp/hcl-lang` `ecfa08c`,
- `tower-lsp-community/tower-lsp-server` `a6ab04d`,
- `oxalica/async-lsp` `e3d479d`,
- `ebkalderon/tower-lsp` `49e1ce5`,
- `rust-lang/rust-analyzer` `f038160`,
- `SamuelMarks/hashicorp-configuration-language-rs` `b665927`,
- `DanielMSchmidt/rust-hcl` `441202d` and `DanielMSchmidt/rust-cty` `5b07f49`.

Verified:
every clone has `git remote set-url --push origin DISABLED` applied before any commit in it.

## The corpus

Verified (`~/temp/agent/hcl-evaluator-2026-09-17/scripts/make-corpus.ts`,
`corpus/list-all.txt`):
2246 files,
1317750 bytes,
every `.tf`,
 `.tofu`,
 `.hcl`,
 and `.tfvars` file in the `opentofu`,
 `hcl`,
 `hcl-rs`,
 `tofu-ls`,
 and `hcl-lang`
clones,
plus three local files:

- `corpus/meow-inventory.hcl`,
  every ` ```hcl ` fence of `doc/planning/monorepo-manager-route-research/stack-declarative-config.md` concatenated,
  12618 bytes,
  which is the closest thing that exists to a real meow configuration;
- `corpus/hetzner.tf`,
   27689 bytes,
   a real OpenTofu module;
- `corpus/comments.hcl`,
   1318 bytes,
   hand written to place a comment in every position the grammar allows:
  leading file comment,
   `#` and `//` line comments,
   `/* */` block comments before blocks,
  after attributes,
   between list elements,
   inside object constructors,
   inside a call argument list,
  between the branches of a conditional,
   inside a `for` expression body,
   and at end of file.

Verified:
`corpus/hetzner-x37.tf`,
 1024529 bytes,
 is `hetzner.tf` repeated 37 times,
used only for throughput measurement.

## Probe results

### Parser acceptance measured against the Go reference

Command,
Verified:

```sh
node scripts/run-box.ts go-oracle-validate localhost/hashvet-rusttest:2 none /w \
  -- sh -c '/t/go-oracle/oracle validate /w/corpus/list-all.txt > /w/data/go-validate.jsonl'
node scripts/run-box.ts roundtrip-hcl-edit localhost/hashvet-rusttest:2 none /w \
  -- sh -c '/t/probe-hcl/release/roundtrip /w/corpus/list-all.txt /w/data/roundtrip-hcl-edit \
       > /w/data/roundtrip-hcl-edit.jsonl'
```

Verified (`data/go-validate.jsonl`,
 `data/roundtrip-hcl-edit.jsonl`,
 `data/probe-others.jsonl`):

- `hclsyntax.ParseConfig` accepts 2185 of 2246 files and rejects 61.
- `hcl-edit` 0.9.7 `parse_body` rejects exactly the same 61 files and accepts exactly the same 2185.
  Zero false rejections and zero false acceptances against the reference.
- `babbel_hcl` 0.2.1 fails on 199 of the 2185 files the reference accepts.
- `hashicorp-configuration-language-rs` (`SamuelMarks`,
   path dependency,
   unpublished)
  fails on 85 of the 2185 and panics on 1.
- `tree-sitter-hcl` 1.1.0 produces zero `ERROR` nodes on all 2185 accepted files,
  and also produces zero `ERROR` nodes on 5 of the 61 files the reference rejects,
  which is expected of an error-recovering grammar and means it cannot be used alone as a validator.

### Round-trip fidelity, the property managed edits depend on

Verified (`data/roundtrip-hcl-edit.jsonl`):
parsing with `hcl-edit` and printing the tree back reproduces the input byte for byte on 2153 of 2185 files
and changes 32.

Verified (`data/roundtrip-hcl-edit-diffs.txt`,
 classified by `scripts/classify-diffs.ts`):
the 32 changed files reduce to two upstream defects.

- 1 file loses comment markers:
  `corpus/comments.hcl` loses 2 comments and gains 3 newline changes.
  The minimal reproduction is `sum = (1 /* inside operands */ + 2)` printing back as `sum = (1 + 2)`,
  and a `for` condition split across lines with `&&` collapsing onto one line,
  dropping `# comment after continued condition`.
- 31 files change heredoc introducers or heredoc body indentation:
  `basic = <<-EOT` prints back as `basic = <<EOT`,
  and a body line indented with a tab prints back with a single space.

Verified (`doc/troubleshooting/hcl-edit-binary-operator-decor.md`,
`doc/troubleshooting/hcl-edit-heredoc-dedent.md`):
both defects were root caused in `hcl-edit` source,
a prototype fix was built for each in a disposable clone,
and both prototypes were verified failing before and passing after,
including the whole upstream workspace test suite.

Unverified:
whether upstream will accept fixes of this shape.
No issue was filed and no upstream comment was posted,
per the task instruction.

### Evaluator conformance measured against `go-cty`

Command,
Verified:

```sh
node scripts/run-box.ts semantics-release localhost/hashvet-rusttest:2 none /w \
  -- /t/probe-hcl/release/semantics /w/corpus/semantics-cases.txt
node scripts/run-box.ts semantics-checked localhost/hashvet-rusttest:2 none /w \
  -- /t/probe-hcl/release-checked/semantics /w/corpus/semantics-cases.txt
node scripts/run-box.ts go-oracle-eval localhost/hashvet-rusttest:2 none /w \
  -- /t/go-oracle/oracle eval /w/corpus/semantics-cases.txt
```

Verified (`data/logs/semantics-release.log` against `data/logs/go-oracle-eval.log`):
`hcl-rs` 0.19.8 `hcl::eval` diverges from `hclsyntax` plus `go-cty` on 12 of 21 cases.

- Object `for` iteration order:
  `hcl-rs` yields `["b", "a"]`,
   the reference yields `["a", "b"]`.
  Verified (`hcl/hclsyntax/spec.md` line 467):
  the specification requires pairs to be visited
  "in the order defined by a lexicographic sort of the attribute names".
  The same divergence appears inside a template `for`:
  `"ba"` against `"ab"`.
- `5 % 0`:
  `hcl-rs` panics with "attempt to calculate the remainder with a divisor of zero",
  the reference returns `5`.
- `1 / 0`:
  `hcl-rs` returns `1.797693134862316e308`,
  the reference returns positive infinity.
  Verified (`hcl/spec.md` lines 249 to 250):
  "The _number_ type also requires representation of both positive and negative infinity."
- `18446744073709551615 + 1`:
  `hcl-rs` returns `0`,
  the reference returns `18446744073709551616`.
  `18446744073709551615 * 2` likewise returns `0`.
- `-9223372036854775808 - 9`:
  `hcl-rs` returns `-9223372036854775808`,
  the reference returns `-9223372036854775817`.
- `20000000000000000001` as a float conversion:
  `hcl-rs` returns `18446744073709551614`,
  the reference returns `20000000000000000001`.
- The literal `123456789012345678901234567890`:
  `hcl-rs` fails to parse ("unexpected token" at column 1),
  the reference evaluates it exactly.
  Verified (`hcl/spec.md` line 238):
  "Integers are represented with at least 256 bits."
- The literal `-9223372036854775809`:
  `hcl-rs` returns `9223372036854775807`,
  a positive number of the wrong magnitude and the wrong sign,
  the reference returns `-9223372036854775809`.
- `0.1 + 0.2`:
  `hcl-rs` returns `0.30000000000000004`,
  the reference returns `0.3`.
- `true ? {a = 1} : "s"`:
  `hcl-rs` returns the object,
  the reference raises "Inconsistent conditional result types".
- `false && undefined_var`:
  `hcl-rs` raises "undefined variable `undefined_var`",
  the reference short circuits to `false`.
- `{for v in [0, -0] : v => v}`:
  `hcl-rs` raises "key `0` already exists",
  the reference returns `{"0": 0, "-0": -0}`.

Verified (`data/logs/semantics-checked.log`):
rebuilt with `overflow-checks = true` and `debug-assertions = true`,
the same binary panics on 4 of those cases instead of returning a wrong value:
remainder by zero,
 `assertion failed: value.is_finite()`,
 add overflow,
 multiply overflow.
The wrong values in the release profile are therefore wrapping and saturation,
not a deliberate numeric model.

Verified (`doc/troubleshooting/hcl-rs-eval-arithmetic-and-iteration-order.md`):
a prototype patch in a disposable clone
(checked arithmetic with a float fallback,
 a divisor-zero error,
 a finite-range guard,
 sorted object iteration)
makes a new 12-case conformance test pass and leaves the upstream workspace suite green.

Verified (`rg 'span' hcl-rs/crates/hcl-rs/src/` returns no matches):
`hcl-rs` evaluation errors carry an owned cloned `Expression`
(`crates/hcl-rs/src/eval/error.rs` line 168,
 `expr: Option<Expression>`)
and no source span,
so a diagnostic built from them cannot point at a line,
 a column,
 or a file.

Verified (`hcl-rs/crates/hcl-edit/src/repr.rs` lines 91 to 94):
`hcl-edit` exposes `fn span(&self) -> Option<Range<usize>>` on `Spanned`,
 `Decorated`,
 and `Formatted`,
documented as "a zero-based start and end byte offset in the input from which this object was parsed".

### Nesting depth and process abort

Command,
Verified:
`node scripts/run-box.ts depth-hcl localhost/hashvet-rusttest:2 none /w -- /t/probe-hcl/release/depth`,
each case in a child process so an abort is observed from outside.

Verified (`data/logs/depth-hcl.log`):
`hcl-edit` parses nested parentheses,
 arrays,
 objects,
 calls,
 and unary operators at depth 1000
(6386 to 8323 microseconds)
and dies with `signal=Some(6)` at depth 5000 for every shape,
 in both the parse and the evaluate phase.

Verified (`data/logs/go-deep.log`,
`node scripts/run-box.ts go-deep localhost/hashvet-rusttest:2 none /w
-- /t/go-oracle/oracle validate /w/corpus/list-deep.txt`):
the Go reference parses the same shapes at depth 1000,
 5000,
 and 20000,
and dies at depth 100000 with
`runtime: goroutine stack exceeds 1000000000-byte limit` followed by `fatal error: stack overflow`.

Verified (`rg 'depth|recursion|MAX_' hcl-rs/crates/hcl-edit/src/parser/*.rs` returns no matches;
the only `depth` match under `hcl/hclsyntax/*.go` is in `walk.go`):
neither implementation has a configurable nesting limit.
The difference is about twentyfold,
 and in kind neither is safe on hostile input without an explicit limit.

### Adversarial robustness of the alternative front ends

Command,
Verified:
`node scripts/run-box.ts robust-run localhost/hashvet-rusttest:2 none /w -- /t/probe-others/release/robust`,
7 shapes times 4 sizes per engine,
 each in a child process with a 20 second wall clock cap.

Verified (`data/logs/robust-run.log`):

- `tree-sitter-hcl` 1.1.0 with `tree-sitter` 0.27.0 survives every case,
  including the input `-+-*a(//` from `tree-sitter-grammars/tree-sitter-hcl` issue 49
  ("Infinite loop on fuzzing input",
   open since 2024-06-22),
  which returns in 69 microseconds with `has_error=true`.
  The worst case measured is 200000 nested interpolations at 1310981 microseconds,
   still returning.
- `hashicorp-configuration-language-rs` aborts with `signal=Some(6)`
  and `fatal runtime error: stack overflow`
  on nested parentheses,
   arrays,
   objects,
   and unclosed parentheses at depth 20000,
  and exits 101 on nested interpolations at depth 10 with
  `panicked at src/parse/parser.rs:1082:44: byte range starts at 2 but ends at 1`.

Verified (`data/ts-hcl-issue-49.json`):
a 2026 comment on issue 49 states the loop no longer reproduces on `main` with tree-sitter 0.27.0,
and the local measurement agrees for the published grammar 1.1.0.

### Formatter behaviour

Command,
Verified:
`node scripts/run-box.ts format-hcl-rs localhost/hashvet-rusttest:2 none /w
-- /t/probe-hcl/release/format /w/corpus/comments.hcl`,
plus `oracle formatall` over the whole corpus.

Verified (`data/logs/format-hcl-rs.log`):

- `hcl-edit` `parse_body` then `to_string` preserves every comment in `comments.hcl`
  except the two lost to the operator defect,
  and performs no formatting at all:
  `mode="0600"` stays unspaced and `path    = "LICENSE"   # destination` keeps its original column.
- `hcl-rs` `hcl::format::to_string` deletes every comment in the file,
  flattens multi-line collections,
  reindents heredoc bodies to column zero,
  and reports `idempotent=false`.

Verified (`data/go-format/`,
 `data/go-validate.jsonl`):
`hclwrite.Format` changes 676 of the 2185 valid files,
leaves 1509 unchanged,
and is idempotent on 2185 of 2185.
Every formatted output still parses.

Verified (`data/comments.tofu-fmt.hcl` against `corpus/comments.hcl`):
`hclwrite.Format` aligns the `=` of consecutive attributes in a body
(`short            = 1` beside `much_longer_name = "two"`),
does not align object constructor items
(`key   = "value"` becomes `key = "value"`),
collapses runs of spaces before a trailing comment to one space,
and inserts the missing spaces in `mode="0600"`.
Verified (`cmp` of two successive runs):
the output is idempotent on that file.

Verified (`opentofu/internal/command/fmt.go` lines 328 to 366):
`tofu fmt` is `hclwrite.ParseConfig` followed by three normalizations
(unwrap a provable single `"${ ... }"` into its expression,
normalize a `variable` block `type` expression,
re-set block labels to the idiomatic quoted form)
and then `f.Bytes()`,
which applies the same token-level formatting.
So `tofu fmt` equals `hclwrite` formatting plus three Terraform-specific rewrites,
none of which apply to meow's block set.

### Speed

Verified (`data/logs/bench-hcl.log`,
 `data/logs/bench-go.log`,
7 runs each,
 median and full band recorded per rule QNB):

- `corpus/meow-inventory.hcl`,
   12618 bytes:
  `hcl-edit` parse median 371.1 microseconds (band 30.5 percent),
  `hcl-rs` parse median 431.8 microseconds,
  `hcl-edit` print median 61.8 microseconds,
  Go `hclsyntax` parse median 1245.5 microseconds,
  Go `hclwrite` format median 968.3 microseconds.
- `corpus/hetzner.tf`,
   27689 bytes:
  `hcl-edit` parse median 638.7 microseconds (band 3.4 percent),
  Go parse median 2452.4 microseconds.
- `corpus/hetzner-x37.tf`,
   1024529 bytes:
  `hcl-edit` parse median 29438.6 microseconds (33.2 MiB per second),
  Go parse median 93931.7 microseconds.

Conclusion,
Verified by the bands:
at realistic configuration sizes every candidate front end completes in well under one millisecond,
and the run-to-run band at 12618 bytes is 30.5 percent,
which is larger than most of the differences between candidates.
Speed decides no adjacent pair in any ranking in this document.

### Static musl binary size

Command,
Verified:
`cargo build --locked --offline --release --target x86_64-unknown-linux-musl --no-default-features --features <one>`
with `opt-level = 3`,
 `lto = true`,
 `codegen-units = 1`,
 `strip = true`,
 `panic = "abort"`,
one feature per library,
each linking a minimal program that actually calls the library.

Verified (`data/sizes.jsonl`,
 bytes of the stripped static-pie executable):

- baseline with `serde` and `serde_json` only: 455424.
- `hcl-edit` 0.9.7:
   799488,
   that is 344064 bytes added.
- `hcl-rs` 0.19.8 with evaluation:
   951072,
   that is 495648 added.
- `tree-sitter` 0.27.0 with `tree-sitter-hcl` 1.1.0:
   639808,
   that is 184384 added.
- `hashicorp-configuration-language-rs`:
   803616,
   that is 348192 added.
- `tokio` baseline with `rt`,
   `io-std`,
   `io-util`,
   `macros`: 562064.
- `lsp-server` 0.10.0 with `lsp-types` 0.97.0:
   729968,
   that is 274544 over baseline.
- `async-lsp` 0.2.4 with `tokio`,
   `tower`,
   `lsp-types` 0.95.1:
   1241984,
   that is 786560 over baseline.
- `tower-lsp-server` 0.23.0 with `tokio`:
   1954704,
   that is 1499280 over baseline.
- `tower-lsp` 0.20.0 with `tokio`:
   2126736,
   that is 1671312 over baseline.
- `lsp-types` 0.97.0 alone: 758528.
   `ls-types` 0.0.6 alone: 770816.
   `gen-lsp-types` 0.11.0 alone: 824064.
  The `gen-lsp-types` probe sets a different capability field because its generated API differs,
  so its number is comparable to within a few kilobytes rather than exactly.
- Diagnostic renderers,
   alone:
  `codespan-reporting` 0.13 at 512768 (57344 added),
  `ariadne` 0.6 at 533248 (77824 added),
  `annotate-snippets` 0.12 at 631552 (176128 added),
  `miette` 7 with `fancy` at 762656 (307232 added).

Verified (`data/sizes.jsonl` first `tower-lsp-server` row,
 status 101):
the first `tower-lsp-server` build failed with `E0063` for a missing `offset_encoding` field in
`InitializeResult`;
the `tower-lsp-server` 0.23.0 measurement is the successful rebuild using `..InitializeResult::default()`.

## Question 1: the evaluator

### What meow has to evaluate

Verified (`doc/planning/monorepo-manager-from-scratch-design.md`,
sections "Declarative configuration" and "Configuration";
`doc/planning/monorepo-manager-route-research/stack-declarative-config.md`):
the configuration is HCL,
 OpenTofu shaped,
 with the full language allowed,
including author-defined functions and recursion,
blocks for `absent`,
 `file`,
 `mirror`,
 `toml_keys`,
 `task`,
 and `check`,
a per-user configuration outside any repository,
and managed edits that must preserve comments.

Verified (`file-enforcer.config.ts`):
the incumbent TypeScript configuration reads files,
globs,
 stats paths,
 parses JSON and TOML,
 hashes with SHA-256,
 renders a YAML template in a loop,
joins,
 sorts,
 dedupes,
 maps,
 filters,
 and spawns an external scanner.
The evaluator therefore needs real data types,
 real iteration,
 and a real function table,
not a key-value reader.

### Options considered

#### Option 1A: use `hcl-rs` `hcl::eval` as published

Design:
depend on `hcl-rs` 0.19.8,
parse to `hcl::Body`,
install functions through `FuncDef`,
evaluate with `Context`.

Pros:
smallest amount of meow code;
published,
 maintained,
 1923525 all-time downloads
(Verified,
 `data/crate-meta.jsonl`);
already understands blocks,
 attributes,
 `for`,
 conditionals,
 templates,
 and function calls.

Cons,
each Verified in "Evaluator conformance measured against `go-cty`":
12 of 21 semantic cases diverge from the reference,
including four that return a silently wrong number
and one that aborts the process;
evaluation errors carry no span,
 so meow cannot satisfy rule DGT with them;
numbers are `u64`,
 `i64`,
 or `f64`
(Verified,
 `hcl-primitives/src/number.rs` lines 34 to 42),
so the specification's 256-bit integers and infinities cannot be represented at all;
the function table is `Func = fn(...)`,
 a bare function pointer,
so an author-defined HCL function,
 which must capture its body and its closure environment,
cannot be installed;
`Evaluate` is a sealed trait,
 so meow cannot extend evaluation without a fork.

Disqualifying problem:
silently wrong numbers and a process abort on `% 0` are correctness defects,
 not feature gaps,
and the missing spans make rule DGT unreachable.

#### Option 1B: fork `hcl-rs` and fix it in the fork

Design:
vendor `hcl-rs` and `hcl-edit`,
apply the prototype patches already built,
add spans to the value tree,
replace the number type,
replace the function table with closures.

Pros:
starts from working `for`,
 template,
 and conditional code;
the three prototype patches are Verified to work and to keep the upstream suite green.

Cons:
the changes that matter are exactly the ones that touch every type:
`hcl::Expression` is a value tree with no place to put a span,
and adding one changes every constructor,
 every `From` impl,
 every serde impl,
 and the public API;
Verified (`rg 'span' hcl-rs/crates/hcl-rs/src/` returns no matches) there is nothing to build on;
the fork then owns a large serde surface meow does not use
(`hcl-rs` exists mainly to serialize and deserialize Rust types to HCL);
divergence from upstream grows with every fix.

Disqualifying problem:
none that is fatal,
but the part being kept (block and expression walking) is the small part,
and the part being rewritten (values,
 numbers,
 functions,
 diagnostics) is the large part.

#### Option 1C: own evaluator over the `hcl-edit` tree

Design:
depend on `hcl-edit` for lexing,
 parsing,
 and the comment-preserving tree;
walk `Body`,
 `Structure`,
 `Attribute`,
 and `Expression` from that tree;
carry `span()` on every node into meow's own value and diagnostic types;
own the number type,
 the function table,
 the scope model,
 and the traversal rules;
re-read numeric literals from the source bytes named by `span()`
so the `u64`/`i64`/`f64` limitation of `hcl-edit` numbers never reaches meow's values.

Pros:
`hcl-edit` acceptance is Verified perfect against the reference on 2246 files;
spans are Verified present on every decorated node;
the same tree is already what managed edits and the formatter need,
so meow keeps one parse of one file for evaluation,
 editing,
 formatting,
 and the language server;
344064 bytes added to a static musl binary,
 Verified;
meow owns the semantics that were Verified wrong in `hcl-rs`,
so conformance is a test suite meow can drive to zero divergences.

Cons:
meow must write everything above the tree:
values,
 numbers,
 function table,
 `for`,
 conditionals,
 templates,
 splats,
 index and traversal rules,
plus the whole diagnostic layer;
Verified,
 `hcl-edit` has no error recovery
(`rg 'recover' hcl-rs/crates/hcl-edit/src/parser/` returns no matches),
so an unparseable file yields one error and no tree,
 which is weaker than the reference for an editor;
Verified,
 `hcl-edit` aborts the process at nesting depth 5000 and offers no depth knob,
so meow must either pre-scan for depth before handing bytes to the parser,
 or vendor a patched parser;
two Verified data-loss defects in write-back must be carried as a patched vendor until upstream fixes them.

#### Option 1D: own lexer, parser, tree, and evaluator

Design:
port the `hclsyntax` grammar to Rust,
own the token stream,
 the CST with trivia,
 error recovery,
 a depth limit,
 and the evaluator.

Pros:
every property meow wants is reachable:
recovery,
 depth limits,
 spans everywhere,
 exact numbers,
 closures,
 deterministic iteration;
no dependency can lose a comment on write-back;
Verified reference sizes to match:
 `hclsyntax` is 14291 lines of Go plus 13561 lines of tests,
`hclwrite` is 3168 plus 7109.

Cons:
the reference suite is the only way to know the port is right,
and the corpus plus the `specsuite` is the only oracle;
the work is strictly larger than option 1C by the size of a correct,
 recovering HCL parser;
Unverified:
 how long that takes.
No estimate is offered here,
 per rule QJ1.

#### Option 1E: port `hcl/v2` and `go-cty` wholesale

Design:
translate `hclsyntax`,
 the `hcl` core,
 `ext/userfunc`,
 `cty`,
 and `cty/function/stdlib` to Rust.

Pros:
maximum fidelity;
the Go tests port with the code.

Cons:
Verified line counts to translate:
`hclsyntax` 14291,
 `hcl` core 2747,
 `hcl/ext` 2882,
 `cty` 20355,
 `cty/function/stdlib` 5960,
so about 46000 lines of Go before tests,
 and about 55000 lines of Go tests;
`cty` carries capsule types,
 marks,
 unknowns,
 and type conversion machinery
that exist for Terraform providers and have no meow consumer;
Unverified but structural:
 a hand port of that size drifts from upstream immediately.

Disqualifying problem:
the majority of the ported surface has no meow consumer.

#### Option 1F: evaluator over a `tree-sitter-hcl` concrete syntax tree

Design:
parse with `tree-sitter` 0.27.0 plus grammar 1.1.0,
walk the CST,
 evaluate from node text.

Pros:
Verified best-in-probe robustness:
survives every adversarial shape at every size tested,
 including issue 49's input;
error recovery is native,
 which is what a language server wants;
smallest measured addition of any front end,
 184384 bytes.

Cons:
Verified,
 it reports zero `ERROR` nodes on 5 files the reference rejects,
so it accepts invalid HCL and cannot be meow's validator;
a `tree-sitter` grammar has no semantic model,
 so every literal,
 escape,
 and heredoc rule is meow's work anyway;
it is a C library compiled by `cc`,
which is a build-system dependency for the aarch64 musl static-pie target that the all-Rust decision avoids;
Verified (`doc/decision/monorepo-manager-all-rust.md`) the all-Rust rule exists precisely to keep C toolchains out;
Verified maintenance risk on the crate:
 crates.io publishes 1.1.0 while the repository tags v1.2.0,
and open issues 77 and 78 describe scanner serialization and a host-dependent `CHAR_MAX` in the external scanner.

Disqualifying problem:
a C dependency in a decision record that forbids C,
 plus acceptance of invalid input.

#### Option 1G: adopt `hashicorp-configuration-language-rs`

Verified:
0 stars,
 0 releases,
 1 contributor,
 4 commits,
 not on crates.io
(`crates.io/api/v1/crates/hashicorp-configuration-language-rs` returns 404),
85 valid files rejected,
 1 panic,
process abort by stack overflow at nesting depth 20000,
a byte-range panic on nested interpolations at depth 10,
and a formatter that differs from `hclwrite` on 1444 of 2185 files and is non-idempotent on those.

Disqualifying problem:
process aborts on inputs a user can type,
 and no published artifact.

#### Option 1H: adopt `babbel_hcl`

Verified:
14 all-time downloads,
199 of 2185 valid files rejected,
repository `clockworkengineer/babbel` with 1 star,
 1 contributor,
 and no releases.

Disqualifying problem:
rejects one valid file in eleven.

#### Option 1I: adopt the `rust-hcl` and `rust-cty` ports

Verified:
`DanielMSchmidt/rust-hcl` contains 280 `todo!()` bodies and `rust-cty` 365;
neither is published to crates.io;
they would have to be git dependencies,
 which the single-file release build cannot take from crates.io.

Disqualifying problem:
unimplemented and unpublished.

#### Option 1J: embed the Go implementation

Design:
ship the Go `hcl` library compiled to Wasm and run it in `wasmtime`,
 or link it through cgo.

Pros:
perfect semantics by construction.

Cons:
Verified (`doc/decision/monorepo-manager-all-rust.md`) the all-Rust decision exists to avoid exactly this;
a Wasm runtime is larger than every option here;
cgo forbids the static musl targets meow ships.

Disqualifying problem:
contradicts a settled decision.

### Ranking for question 1

1.  Option 1C,
     own evaluator over the `hcl-edit` tree.
2.  Option 1D,
     own lexer,
     parser,
     and evaluator.
3.  Option 1B,
     fork `hcl-rs`.
4.  Option 1F,
     evaluator over `tree-sitter-hcl`.
5.  Option 1E,
     port `hcl/v2` and `go-cty`.
6.  Option 1A,
     `hcl-rs` as published.
7.  Option 1G,
     `hashicorp-configuration-language-rs`.
8.  Option 1H,
     `babbel_hcl`.
9.  Option 1I,
     the `rust-hcl` and `rust-cty` ports.
10. Option 1J,
     embedding Go.

Adjacent-pair reasons.

- 1C over 1D:
  `hcl-edit` is Verified to accept exactly the files the reference accepts and reject exactly the ones it rejects,
  on 2246 files,
  so the parser is the one part meow does not have to prove correct;
  everything 1D would add above that is the same work 1C already does.
- 1D over 1B:
  both write the value,
   number,
   function,
   and diagnostic layers,
  and 1D gets error recovery,
   a depth limit,
   and a formatter-friendly token stream for that price,
  while 1B inherits `hcl-rs`'s serde surface and its sealed traits and still has to add spans to a tree
  that Verified has none.
- 1B over 1F:
  1B starts from a parser that is Verified exact against the reference,
  while 1F starts from a parser that is Verified to accept 5 invalid files and needs a C toolchain
  that `doc/decision/monorepo-manager-all-rust.md` rules out.
- 1F over 1E:
  1F is a working,
   maintained,
   robust parser today,
  while 1E is about 46000 lines of Go to translate before the first configuration evaluates,
  most of it `cty` machinery with no meow consumer.
- 1E over 1A:
  1E is slow and large but converges on correct semantics,
  while 1A is Verified to return wrong numbers silently and to abort on `% 0`,
  and no amount of meow code on top can fix a number type that cannot hold the value.
- 1A over 1G:
  `hcl-rs` is published,
   maintained,
   and never aborted in the corpus run,
  while `hashicorp-configuration-language-rs` aborts by stack overflow on inputs a user can type.
- 1G over 1H:
  both are single-author and unproven,
  but `babbel_hcl` rejects 199 valid files against 85,
   and has 14 downloads against an unpublished crate that at
  least builds and formats.
- 1H over 1I:
  `babbel_hcl` parses most files;
  `rust-hcl` has 280 `todo!()` bodies and cannot be depended on from crates.io at all.
- 1I over 1J:
  an unfinished Rust port can be finished;
  embedding Go contradicts a settled decision record and the static musl targets.

### The recommended design in detail

Recommendation:
option 1C.
meow depends on `hcl-edit` for lexing,
 parsing,
 and the comment-preserving tree,
and owns everything above it.
Until the two Verified write-back defects are fixed upstream,
meow carries `hcl-edit` as a patched vendored crate with the two prototype patches already built and verified.

#### Values and numbers

Unverified design,
 following `hcl/spec.md` lines 232 to 250:
meow's value type is `Null`,
 `Bool`,
 `Number`,
 `String`,
 `Tuple`,
 `Object`,
where `Number` is an arbitrary-precision decimal or rational with explicit positive and negative infinity
and no NaN.
Integers must hold at least 256 bits,
and a literal that cannot be represented exactly must raise an error rather than round.
Verified as the reason:
`hcl-edit` numbers are `u64`,
 `i64`,
 or `f64`,
so meow re-reads the literal's bytes through `span()` and parses them itself.
The comparison and equality rules follow the reference,
where `1` and `1.0` compare equal
(Verified:
 both implementations already agree on that case).

#### Diagnostics, spans, and rule DGT

Unverified design:
one `Diagnostic` type with a severity,
 a primary span,
 zero or more secondary spans,
 a summary,
 and a detail body.
Every span is a byte range into a named source file,
obtained from `hcl-edit`'s `span()` and never discarded.
Rule DGT requires naming the affected input and the calls plainly,
so every diagnostic carries three things:
the file and line and column of the expression that failed,
the chain of block labels and attribute names that leads to it
(the reference does this as "EVAL DIAGNOSTICS for outer.label.inner",
Verified in `data/logs/go-oracle-eval.log`),
and,
 for a failure inside a `for` or a function call,
the bound values that produced it
(the reference prints "with s as \"a\"",
 Verified in the same log).
Rule DNL applies to the wording:
name the operation and the evidence,
 never characterize the author.

Renderer choice:
Verified sizes are 57344 bytes for `codespan-reporting`,
 77824 for `ariadne`,
176128 for `annotate-snippets`,
 307232 for `miette` with `fancy`.
No recommendation is made here because meow's terminal output layer is not designed yet;
this is recorded as a deferred choice,
 not a silent one,
 and no vet report is opened for it.

#### Determinism

Unverified design,
 Verified requirement:
object and map iteration is a lexicographic sort of the keys
(`hcl/hclsyntax/spec.md` line 467),
which meow implements by sorting at the iteration boundary rather than by choosing an ordered map,
so that the source order of an object literal is still available to the formatter and to managed edits.
Function evaluation is pure by construction except for the declared-read functions of question 2,
so a configuration evaluates to the same value given the same inputs,
and the daemon can cache on the recorded reads.

#### Cycles, recursion, and limits

Unverified design,
 Verified motivation:
both `hcl-edit` and the Go reference abort the process on deep nesting,
at depth 5000 and depth 100000 respectively,
so meow imposes its own limits before either can be reached.

- A syntactic depth limit is checked while walking the tree,
  and,
   because `hcl-edit` itself aborts during parsing,
  a cheap pre-scan of brackets and parentheses runs over the bytes before the parser is called.
  A file that exceeds the limit gets a diagnostic naming the file,
   the offset,
   and the limit.
- Value references form a dependency graph over attributes and locals;
  evaluation is a topological walk with an explicit visiting set,
  so a cycle is a diagnostic that names every member of the cycle in source order,
   not a hang.
- Author-defined functions may recurse.
  Recursion is bounded by a call-depth limit and by a total evaluation fuel budget,
  both configurable and both reported by name when exceeded.
  Verified precedent for the shape of the limit:
  OpenTofu bounds `templatefile` recursion at depth 1024
  (`opentofu/internal/lang/funcs`),
  and its symbol libraries reject recursion outright;
  meow's settled requirement allows recursion,
   so meow needs the budget instead of the ban.

#### Lazy or eager

Unverified design:
lazy per attribute,
 eager within an attribute.
An attribute's value is computed when something asks for it,
 memoized after,
and the dependency edges recorded during that computation are what the daemon caches on.
Reasons:
a repository configuration that declares many `file` and `mirror` blocks should not read every managed file
to answer a question about one of them;
laziness is also what makes the reference's `&&` short circuit correct,
which is one of the 12 Verified divergences in `hcl-rs`.
Blocks,
 by contrast,
 are enumerated eagerly,
because meow must know the full set of declared files before it can report a file that is present but undeclared.

## Question 2: the function library

### What the incumbent configuration actually does

Verified (`file-enforcer.config.ts`,
 2329 lines,
 counted with `rg --only-matching ... | sort | uniq --count`):
`toml` appears 24 times,
`map` 21,
`every` 12,
`Object.keys` 8,
`join` 8,
`glob` 7,
`cwd` 7,
`resolve` 6,
`includes` 6,
`filter` 6,
`startsWith` 5,
`Object.entries` 4,
`JSON.stringify` 4,
`JSON.parse` 4,
`some` 3,
`lstat` 3,
`chmod` 3,
`spawn` 2,
`replaceAll` 2,
`readFile` 2,
`mkdir` 2,
`endsWith` 2,
`createHash` 2,
`yaml` 1,
`stat` 1,
`sort` 1,
`Object.values` 1,
`flatMap` 1.

Verified (`file-enforcer.config.ts` lines 1090 to 1106):
the only hashing is `createHash('sha256').update(content, 'utf8').digest('hex')` over canonical Markdown content,
used to decide mirror ownership.
Verified (line 1791):
one YAML file is rendered from a template.
Verified (line 115 and the `Cargo.toml` glob):
TOML reading is a property lookup in manifests,
 which is what the `toml_keys` block exists for.

Three of those operations are not function work at all.
Verified by their names:
`chmod` and `mkdir` mutate the filesystem,
and `spawn` runs an external scanner.
Unverified design:
these belong to the `file`,
 `task`,
 and `check` blocks,
where the daemon can schedule them,
 sandbox them,
 and report them,
and a configuration language function must never perform them.

### Reference sets measured

Verified (`rg --only-matching '^var ([A-Za-z0-9]+)Func = function.New' go-cty/cty/function/stdlib/*.go | wc --lines`):
the HCL standard library,
 meaning `go-cty`'s `function/stdlib`,
 defines 80 functions.

Verified (`rg --count-matches '^\t\t"[a-z_0-9]+":' opentofu/internal/lang/functions.go`):
OpenTofu's base table has 133 entries,
listed in full in `data/` and reproduced here by name:
`abs abspath alltrue anytrue assumeequal assumelistlength ... zipmap`.

Verified (`opentofu/internal/lang/functions.go` lines 23 to 28):
OpenTofu marks exactly three functions impure:
`bcrypt`,
 `timestamp`,
 `uuid`.

Verified (`opentofu/internal/lang/funcs/filesystem.go` line 74 and `ErrorTemplateRecursionLimit`):
`templatefile` may call itself,
 bounded at a default depth of 1024.

### Options considered

#### Option 2A: adopt the OpenTofu table wholesale

Pros:
an author who knows OpenTofu knows every name;
the "OpenTofu-shaped" promise is kept literally.

Cons:
Verified,
 the table contains `bcrypt`,
 `uuid`,
 `timestamp`,
 `plantimestamp`,
 `rsadecrypt`,
the `cidr*` family,
 the `assume*` family,
 `sensitive`,
 `nonsensitive`,
 `issensitive`,
and `ephemeralasnull`,
none of which has a meow consumer;
three are impure by OpenTofu's own list,
 and an impure function makes a cached evaluation unreproducible.

#### Option 2B: a curated OpenTofu-named set plus meow additions

Design:
take every OpenTofu name whose semantics meow can implement exactly and that a meow configuration can use,
drop the provider,
 Terraform,
 and impure families,
add the few functions the meow blocks need,
and keep every retained name and signature identical to OpenTofu's.

Retained by category,
all Verified to exist in the OpenTofu table:
strings (`chomp`,
 `endswith`,
 `format`,
 `formatlist`,
 `indent`,
 `join`,
 `lower`,
 `replace`,
 `split`,
`startswith`,
 `strcontains`,
 `strrev`,
 `substr`,
 `title`,
 `trim`,
 `trimprefix`,
 `trimspace`,
 `trimsuffix`,
`upper`,
 `regex`,
 `regexall`),
collections (`alltrue`,
 `anytrue`,
 `chunklist`,
 `coalesce`,
 `coalescelist`,
 `compact`,
 `concat`,
 `contains`,
`distinct`,
 `element`,
 `flatten`,
 `index`,
 `keys`,
 `length`,
 `lookup`,
 `matchkeys`,
 `merge`,
 `one`,
 `range`,
`reverse`,
 `setintersection`,
 `setproduct`,
 `setsubtract`,
 `setunion`,
 `slice`,
 `sort`,
 `sum`,
 `transpose`,
`values`,
 `zipmap`),
numbers (`abs`,
 `ceil`,
 `floor`,
 `log`,
 `max`,
 `min`,
 `parseint`,
 `pow`,
 `signum`),
encodings (`base64decode`,
 `base64encode`,
 `csvdecode`,
 `jsondecode`,
 `jsonencode`,
 `textdecodebase64`,
`textencodebase64`,
 `urldecode`,
 `urlencode`,
 `yamldecode`,
 `yamlencode`),
hashes (`base64sha256`,
 `base64sha512`,
 `md5`,
 `sha1`,
 `sha256`,
 `sha512`),
paths (`abspath`,
 `basename`,
 `dirname`,
 `pathexpand`),
type conversion (`tobool`,
 `tolist`,
 `tomap`,
 `tonumber`,
 `toset`,
 `tostring`,
 `try`,
 `can`),
and the declared-read family (`file`,
 `filebase64`,
 `fileexists`,
 `fileset`,
 `filemd5`,
 `filesha1`,
`filesha256`,
 `filesha512`,
 `filebase64sha256`,
 `filebase64sha512`,
 `templatefile`).

Added for meow,
Unverified because they do not exist upstream:

- `tomldecode`,
   because the `toml_keys` block reads manifests and OpenTofu has no TOML function
  (Verified:
   no `toml` entry in the 133-name table);
- `filetype(path)` returning one of `"file"`,
   `"directory"`,
   `"symlink"`,
   `"absent"`,
  because the incumbent calls `lstat` three times to distinguish those cases
  and `fileexists` alone cannot;
- `jsonencode` with an explicit indent argument,
   or a sibling name,
  because the incumbent uses `JSON.stringify` with indentation 4 times
  and OpenTofu's `jsonencode` emits compact JSON only;
- `glob(pattern)` as an alias shape over `fileset`,
   if the `fileset(root, pattern)` split proves awkward
  for the 7 glob call sites.

Dropped,
each with a Verified reason:
`bcrypt`,
 `uuid`,
 `timestamp`,
 `plantimestamp` because they are impure and break reproducible evaluation;
`rsadecrypt`,
 `sensitive`,
 `nonsensitive`,
 `issensitive`,
 `ephemeralasnull` because they serve Terraform state;
the `cidr*` family because no meow block takes a network;
the `assume*` family because it exists for OpenTofu's static type assertions;
`convert` and `type` because they expose `cty`'s type system,
 which meow does not have.

Pros:
every retained name behaves exactly as an OpenTofu author expects;
the set is small enough to test exhaustively against `go-cty`'s own test vectors
(Verified:
 8229 lines of stdlib tests exist to port assertions from);
determinism is a property of the whole table rather than a caveat.

Cons:
four meow-only names diverge from OpenTofu,
 so the promise is "shaped like" and not "identical to";
`try` and `can` require the evaluator to catch a diagnostic and continue,
 which constrains the error type.

#### Option 2C: the minimal set the incumbent needs

Pros:
smallest surface,
 fastest to prove correct.

Cons:
meow serves other repositories too (settled requirement),
so a set fitted to one repository's configuration is a set that a second repository immediately outgrows.

#### Option 2D: no built-ins, everything author-defined

Pros:
the smallest possible built-in surface;
uses the settled allowance for author-defined functions.

Cons:
`sort`,
 `format`,
 `regex`,
 and the encodings cannot be written in HCL at all;
Verified,
 the reference implementations of those live in 5960 lines of Go.

#### Option 2E: depend on an existing Rust port of the function library

Verified:
`rust-cty` contains 369 `todo!()` bodies and is not published to crates.io.

Disqualifying problem:
unimplemented and unpublishable into a crates.io release.

#### Option 2F: expose a general `exec()` function

Pros:
covers the incumbent's scanner `spawn` without a new block.

Cons:
an arbitrary subprocess makes every evaluation non-deterministic,
makes the read set unknowable,
and makes the daemon's cache unsound;
Verified as the shape of the problem:
OpenTofu keeps its three impure functions on a named list precisely so it can force them to unknown.

Disqualifying problem:
it destroys the property that question 2 exists to protect.

### Ranking for question 2

1.  Option 2B,
     curated OpenTofu-named set plus meow additions.
2.  Option 2A,
     the OpenTofu table wholesale.
3.  Option 2C,
     the minimal set.
4.  Option 2D,
     author-defined only.
5.  Option 2E,
     an existing Rust port.
6.  Option 2F,
     a general `exec()`.

Adjacent-pair reasons.

- 2B over 2A:
  2A includes three functions OpenTofu itself marks impure,
  and a dozen families with no meow consumer,
  so 2A buys name compatibility at the cost of the determinism the daemon's cache depends on.
- 2A over 2C:
  familiarity to an OpenTofu author is the stated shape of the language,
  and a minimal set makes every second repository write its own `sort`.
- 2C over 2D:
  a small correct built-in set beats HCL-level reimplementations of functions
  that Verified need 5960 lines of reference code.
- 2D over 2E:
  author-defined functions work today;
  `rust-cty` has 369 unimplemented bodies and no crates.io release.
- 2E over 2F:
  an unfinished port can be finished,
  while `exec()` makes caching unsound by construction.

### How I/O functions declare their reads

Unverified design,
Verified motivation
(the daemon needs to know when a cached evaluation is stale,
and OpenTofu's own answer is a list of impure names,
 which is coarser than meow needs):

- Every function is declared pure or declared-read.
  A pure function receives only its arguments.
  A declared-read function additionally receives a read sink.
  There is no ambient filesystem access in any function body,
  which makes the split checkable by construction rather than by review.
- A read sink records one of:
  `File { path, digest }`,
  `Absent { path }`,
  `Directory { path, entry_names_digest }`,
  `Glob { root, pattern, matched_paths }`,
  `Env { name, value_digest }`.
  `Absent` matters as much as `File`:
  a configuration that branches on `fileexists` must re-evaluate when the missing file appears.
  `Glob` records the matched set and the roots walked,
  so creating a new matching file invalidates even though no recorded file changed.
- The read set is returned beside the value and is part of the evaluation result,
  so the daemon watches exactly what was read,
  and the cache key is the digest of the configuration files plus the digest of the read set
  plus the function table version.
- An author-defined function may call declared-read functions.
  Its reads propagate through the same sink,
  so the read set stays complete without the author declaring anything.
- `templatefile` is a declared read of the template plus the reads its body performs,
  bounded by the same call-depth budget as any other recursion.
  Verified precedent:
   OpenTofu bounds it at 1024.

## Question 3: the formatter

### What the reference formatter does, measured

Verified (`data/go-format/` produced by `oracle formatall`,
 tallied from `data/go-validate.jsonl`):
`hclwrite.Format` changes 676 of the 2185 valid corpus files,
leaves 1509 byte identical,
is idempotent on 2185 of 2185,
and every output still parses.

Verified (`data/comments.tofu-fmt.hcl` against `corpus/comments.hcl`,
 and `cmp` of two successive runs):
on the comment-heavy fixture it aligns the `=` of consecutive body attributes,
does not align object constructor items,
reduces runs of spaces before a trailing comment to one space,
adds the missing spaces around `=` in `mode="0600"`,
keeps every comment in place,
and is idempotent.

Verified (`opentofu/internal/command/fmt.go` lines 328 to 366):
`tofu fmt` is that same formatting plus three Terraform-specific normalizations
(unwrapping a provable single `"${ ... }"`,
 normalizing a `variable` block `type`,
 re-setting block labels),
none of which applies to meow's block set.
So "match `tofu fmt`" and "match `hclwrite`" are the same target for meow.

### Options considered

#### Option 3A: own formatter over the `hcl-edit` tree

Design:
the formatter rewrites only the decoration (whitespace and comments) attached to each node,
never the node text,
and emits through the same tree used for evaluation and for managed edits.
The conformance target already exists:
`data/go-format/` holds the reference output for 2185 real files,
so meow's formatter is correct when it reproduces those bytes and is idempotent on all of them.

Pros:
one parse serves evaluation,
 formatting,
 managed edits,
 and the language server's formatting request;
comments are nodes in the tree,
 so "preserve comments" is structural rather than best effort;
Verified,
 `hcl-edit` prints back 2153 of 2185 files byte identically today,
so the starting point for a formatter is a correct printer.

Cons:
Verified,
 two upstream defects currently lose comments and heredoc introducers,
so the formatter is only safe on a patched `hcl-edit`;
decoration is stored as raw strings
(Verified:
 `hcl-edit` `Decor` holds raw prefix and suffix text),
so alignment work means rewriting those strings rather than laying out a token stream,
which is more awkward than the reference's approach.

#### Option 3B: own token-level formatter over an owned lexer

Design:
follow `hclwrite`:
lex to tokens,
 compute the format in terms of tokens and columns,
 print.
Verified reference size:
 `hclwrite` is 3168 lines of Go with 7109 lines of tests.

Pros:
the algorithm to copy is exactly the reference algorithm;
alignment is natural in a token model.

Cons:
it requires meow to own the lexer,
 which is option 1D's cost;
two representations of the same file then exist unless the evaluator also moves to the token model.

#### Option 3C: use `hcl-rs`'s `hcl::format`

Verified (`data/logs/format-hcl-rs.log`):
it deletes every comment in the fixture,
flattens multi-line collections,
reindents heredoc bodies to column zero,
and reports `idempotent=false`.

Disqualifying problem:
it destroys comments,
which the settled requirement forbids.

#### Option 3D: treat `hcl-edit`'s `to_string` as the formatter

Verified:
it performs no formatting at all,
leaving `mode="0600"` unspaced.

Disqualifying problem:
a `meow fmt` that formats nothing is a promise the tool does not keep.

#### Option 3E: adopt the `SamuelMarks` formatter

Verified (`data/probe-others.jsonl`):
against `hclwrite` output on the 2185 valid files it matches and is idempotent on 533,
differs but is idempotent on 207,
differs and is not idempotent on 1444,
and errors on 1.

Disqualifying problem:
a formatter that is not idempotent rewrites the repository on every run.

#### Option 3F: shell out to `tofu fmt`

Disqualifying problem:
the settled requirement is a single-file binary,
and this makes formatting depend on an OpenTofu installation that meow does not ship.

#### Option 3G: ship no formatter

Pros:
nothing to get wrong;
managed edits still preserve comments because that is the tree's property,
 not the formatter's.

Cons:
the language server cannot answer `textDocument/formatting`,
and a repository manager that writes configuration cannot normalize what it writes.

### Ranking for question 3

1.  Option 3A,
     own formatter over the `hcl-edit` tree.
2.  Option 3B,
     own token-level formatter.
3.  Option 3G,
     no formatter.
4.  Option 3D,
     `to_string` as the formatter.
5.  Option 3F,
     shell out to `tofu fmt`.
6.  Option 3E,
     the `SamuelMarks` formatter.
7.  Option 3C,
     `hcl::format`.

Adjacent-pair reasons.

- 3A over 3B:
  both write a formatter,
   but 3A reuses the tree that evaluation and managed edits already need,
  while 3B needs an owned lexer first.
- 3B over 3G:
  a token formatter is work with a Verified reference and a Verified 2185-file conformance corpus,
  while no formatter leaves `textDocument/formatting` unanswerable.
- 3G over 3D:
  no formatter is honest;
  a formatter that changes nothing looks like a formatter and is not one.
- 3D over 3F:
  a no-op inside the single binary is better than a correct result that requires a second program
  the release does not ship.
- 3F over 3E:
  `tofu fmt` is correct where it is available;
  a non-idempotent formatter is wrong everywhere and churns files on every run.
- 3E over 3C:
  churning whitespace is recoverable;
  deleting every comment is not.

### The recommended design in detail

Recommendation:
option 3A.

Unverified design points,
with the Verified reference behaviour each one has to match:

- Indentation is two spaces per nesting level.
- Consecutive attributes in a body align their `=`,
  and the run is broken by a blank line,
   by a nested block,
   and by an attribute whose value spans lines.
  Verified:
   the reference aligns `short` with `much_longer_name` across an intervening comment line.
- Object constructor items are not aligned.
  Verified:
   `key   = "value"` becomes `key = "value"`.
- Exactly one space precedes a trailing comment.
- The formatter never moves a comment across a token boundary,
  never deletes a comment,
  and never changes a heredoc introducer or body.
  Verified as the failure mode to guard against:
  both are the upstream defects recorded in `doc/troubleshooting/`.
- Formatting is idempotent,
  tested by formatting `data/go-format/` output again and comparing bytes,
  which the reference passes 2185 times out of 2185.
- Line endings:
  the settled requirement says CRLF need not be preserved,
  so the formatter writes `\n`.
  Unverified:
  what `hclwrite` does with CRLF input,
   which was not measured.

## Question 4: the language server

### What the four candidate frameworks are, measured

Verified at the consumer boundary
(`scripts/lsp-client.ts` driving each built binary over stdio,
results in `data/lsp-consumer-*.json`):
all four frameworks answered `initialize`,
published diagnostics after `didOpen`,
and answered `textDocument/completion`,
 `hover`,
 `definition`,
 `formatting`,
 and `shutdown`,
then exited 0.
Seven messages each,
 exit code 0 each.
So none of them is disqualified by failing to speak the protocol.

Verified sizes,
 static musl,
 stripped,
 over a 455424-byte baseline:
`lsp-server` with `lsp-types` adds 274544;
`async-lsp` with `tokio` and `tower` adds 786560;
`tower-lsp-server` with `tokio` adds 1499280;
`tower-lsp` with `tokio` adds 1671312.

Verified upstream suites,
 run in the container:

- `tower-lsp-server` `a6ab04d`:
   44 tests pass with `runtime-tokio`,
   44 pass with `runtime-agnostic`,
   0 fail.
- `async-lsp` `e3d479d`:
   4 tests pass with `--all-features --all-targets`,
   plus 2 doc tests,
   0 fail.
- `lsp-server` 0.10.0:
   `cargo test --lib` gives 10 passed,
   0 failed.
  Verified quirk:
   the published crate's integration tests and examples do not build from the crate alone
  (`unresolved import lsp_types`),
  so the library tests were run after adding `gen-lsp-types` as a dev dependency to a scratch copy.
- `tower-lsp` `49e1ce5`:
   every attempted feature combination fails to compile
  (`--all-features`,
   `runtime-tokio`,
   `runtime-agnostic`,
   and both with `proposed`),
  with `WorkspaceDiagnosticRefresh` not in scope.

Verified maintenance:

- `tower-lsp-community/tower-lsp-server`:
   pushed 2026-09-11,
   224 stars,
   2 open issues,
  releases through `v0.24.0-rc.1` on 2026-09-11 and `v0.23.0` on 2025-12-07.
- `oxalica/async-lsp`:
   pushed 2026-08-09,
   178 stars,
   4 open issues,
  `v0.2.4` on 2026-04-24,
   contributor list 158 commits by one person and 1 each by three others.
- `ebkalderon/tower-lsp`:
   pushed 2024-08-15,
   1362 stars,
   41 open issues,
  last release `v0.20.0` on 2023-08-11.
- `rust-lang/rust-analyzer`,
   which contains `lsp-server`:
   pushed 2026-09-17,
   releases weekly.

Verified source surface (`scripts/lsp-source-audit.ts`):
`lsp-server` is 1129 lines in 6 files with 10 tests and zero `unsafe`;
`async-lsp` is 2954 lines in 11 files with 3 tests,
 3 `unsafe`,
 and 3 `catch_unwind`;
`tower-lsp` is 5403 lines with 14 tests;
`tower-lsp-server` is 5949 lines with 14 tests.
Verified for all four:
zero matches for `position_encoding`,
 `positionEncoding`,
 or `PositionEncoding`,
so UTF-16 position conversion is meow's work whichever framework is chosen.
Verified:
`cancelRequest` appears 11 times in `tower-lsp-server`,
 9 in `tower-lsp`,
 3 in `async-lsp`,
 and 0 in `lsp-server`,
so cancellation is also meow's work if `lsp-server` is chosen.

### Options considered

#### Option 4A: `lsp-server` plus a types crate

Design:
a hidden `meow lsp` subcommand runs a synchronous main loop over stdio,
dispatching to a worker pool that owns the parsed trees and the evaluator,
the way rust-analyzer does.

Pros:
Verified smallest addition of the four,
 274544 bytes;
Verified maintained inside a repository that releases weekly;
Verified smallest source surface,
 1129 lines,
 which meow can read in full;
no async runtime enters the binary,
 so the single binary keeps one execution model.

Cons:
Verified,
 no `cancelRequest` support,
 so meow writes cancellation;
Verified,
 the published crate's tests and examples do not build standalone;
the types crate is a separate decision
(Verified sizes:
 `lsp-types` 0.97.0 alone 758528,
 `ls-types` 0.0.6 alone 770816,
`gen-lsp-types` 0.11.0 alone 824064),
and Verified,
 `lsp-types` upstream is `gluon-lang/lsp-types` whose last commit in the clone is 2024-06-04,
with `ls-types` and `gen-lsp-types` as the maintained successors.
That sub-decision is settled rather than asked,
in `doc/audit/tech-meow-language-server-framework-vet-2026-09-17.md`,
 "Protocol types":
`gen-lsp-types` 0.11.0,
because rust-analyzer depends on it beside `lsp-server`
and it was regenerated against the specification 12 times in the past year
while `lsp-types` has had no release since 2024-06-04.

#### Option 4B: `tower-lsp-server`

Pros:
Verified most active of the frameworks,
 44 tests passing on both runtimes;
Verified most `cancelRequest` handling;
a trait-based server is the least code for meow to write.

Cons:
Verified largest practical cost after `tower-lsp`,
 1499280 bytes,
 because it brings `tokio`;
an async runtime in a tool whose other subcommands are synchronous.

#### Option 4C: `async-lsp`

Pros:
Verified middle size,
 786560 bytes;
tower layers give cancellation,
 concurrency,
 and tracing as composable middleware.

Cons:
Verified 3 tests in the whole crate and a single maintainer;
Verified quirk found while testing:
a `Router` that omits `request::Shutdown` answers `-32601 No such method shutdown`,
because `LifecycleLayer` tracks state and forwards rather than answering,
which is a foot-gun meow would have to remember.

#### Option 4D: `tower-lsp`

Verified:
does not compile at HEAD in any feature combination tried,
last release 2023-08-11,
41 open issues,
and the community fork exists because of it.

Disqualifying problem:
an unmaintained dependency that does not build from source is one meow cannot fix without forking.

#### Option 4E: hand-rolled JSON-RPC over stdio

Pros:
zero dependency;
Verified as small in principle:
 `lsp-server` does the whole job in 1129 lines.

Cons:
meow would rewrite exactly those 1129 lines,
 with 10 fewer tests than they have.

#### Option 4F: a separate `meow-lsp` binary

Disqualifying problem:
the settled requirement is a single-file binary.

#### Option 4G: no language server, rely on editors' `tree-sitter` grammar

Pros:
syntax highlighting already exists in editors that bundle `tree-sitter-hcl`.

Cons:
no diagnostics,
 no completion of meow's block schema,
 no go-to-definition;
Verified,
 `tofu-ls` cannot stand in:
it is 27675 lines built on `hcl-lang`'s 17375-line schema layer,
 both shaped for Terraform's schema,
not meow's blocks.

### Ranking for question 4

1.  Option 4A,
     `lsp-server` plus a types crate.
2.  Option 4B,
     `tower-lsp-server`.
3.  Option 4C,
     `async-lsp`.
4.  Option 4E,
     hand-rolled JSON-RPC.
5.  Option 4G,
     no language server.
6.  Option 4D,
     `tower-lsp`.
7.  Option 4F,
     a separate binary.

Adjacent-pair reasons.

- 4A over 4B:
  4B costs 1224736 more bytes and brings an async runtime into a tool with no other async subcommand,
  and buys request cancellation that meow can write against a synchronous loop it already controls.
- 4B over 4C:
  Verified,
   `tower-lsp-server` has 44 passing tests and a live release train,
  while `async-lsp` has 4 tests,
   one maintainer,
   and a shutdown foot-gun found in the first hour of use.
- 4C over 4E:
  `async-lsp` is a working,
   tested library;
  hand-rolling repeats known work.
- 4E over 4G:
  framing code meow writes still yields diagnostics,
   completion,
   and go-to-definition;
  no server yields none.
- 4G over 4D:
  no dependency is better than one that Verified does not compile at HEAD and has no release since 2023.
- 4D over 4F:
  an unmaintained dependency is a maintenance problem;
  a second binary breaks the distribution requirement outright.

### The recommended design in detail

Recommendation:
option 4A.

- Transport:
  `meow lsp` on stdio,
   a hidden subcommand of the same binary,
  so editors launch the tool they already have and the release stays one file.
  Verified cost:
   274544 bytes.
- Threading:
  one reader thread,
   one main loop,
   one worker pool.
  Document state is the parsed tree plus the source text;
  an edit reparses the file it touched.
  Verified as affordable:
  a 12618-byte configuration parses in a median of 371.1 microseconds,
  so full reparse per keystroke is well inside an editor's budget and no incremental parser is needed.
- Diagnostics:
  the server runs the same evaluator as the CLI,
  so a diagnostic in the editor is the diagnostic the command line prints,
   with the same spans.
  Parse errors come from `hcl-edit`,
   which Verified stops at the first error,
  so on a parse failure the server reports that error and keeps serving completion and hover
  from the last tree that parsed.
  Unverified:
  whether that degradation is acceptable in practice,
  or whether it eventually forces an error-recovering parser,
   which is option 1D's territory.
- Completion:
  block types at body level,
   attribute names inside a block,
   function names with signatures,
  values of enumerated attributes,
   `for` variables in scope,
   and references to declared locals and blocks.
  The schema comes from meow's own block definitions,
  which meow owns because it defines the blocks;
  no `hcl-lang` equivalent is needed.
- Hover:
  the attribute's documentation and type,
  the function's signature and documentation,
  and,
   for a pure and already-computed local,
   its evaluated value.
- Go-to-definition:
  a reference resolves to the attribute or block that declares it,
  and a `file` block path resolves to the file on disk.
- Formatting:
  `textDocument/formatting` returns a single full-document edit produced by the question 3 formatter.
- Cancellation:
  meow implements `$/cancelRequest` itself with a cancellation flag per request identifier,
  checked at evaluation boundaries.
  Verified as necessary:
   `lsp-server` has no `cancelRequest` handling.
- Position encoding:
  meow converts byte offsets from `span()` to UTF-16 code units for every range it emits,
  and back for every position it receives.
  Verified as necessary:
   none of the four frameworks does this.

## Interactions with settled requirements

- Single-file binary:
  the recommended set adds 344064 bytes for `hcl-edit` and 274544 bytes for `lsp-server` with `lsp-types`,
  both Verified on a stripped static musl build,
  so the four answers together cost about 0.6 MiB of the release artifact before meow's own code.
- Published to crates.io:
  every recommended dependency is published there.
  Verified exclusions that this forces:
  `rust-hcl`,
   `rust-cty`,
   and `hashicorp-configuration-language-rs` are git-only,
  and `hashicorp-configuration-language-rs` returns 404 from the crates.io API.
- All-Rust:
  the recommendation adds no C toolchain.
  Verified as the reason `tree-sitter-hcl` was ranked below the alternatives despite the best robustness
  numbers in the probe.
- Per-user configuration outside the repository:
  the evaluator must carry a base directory per configuration file,
  because `file`,
   `fileset`,
   and `templatefile` resolve relative to it
  (Verified shape:
   OpenTofu threads `s.BaseDir` into the same functions).
  The daemon's read set must cover the per-user configuration's reads as well,
  and the cache key must include both configurations' digests,
  or a change to the per-user file will not invalidate a repository's cached evaluation.
  Discovery and precedence of that file are separate work and are not designed here.
- meow serves other repositories:
  the function set is chosen for that reason in option 2B rather than fitted to `file-enforcer.config.ts`.
- Managed edits preserve comments,
  untouched bytes and CRLF need not be preserved:
  this is why comment preservation is a tree property in option 1C and question 3,
  and why the two Verified `hcl-edit` write-back defects are treated as blocking rather than cosmetic.

## Risks

- Carrying a patched `hcl-edit` means every upstream release has to be re-patched and re-measured.
  Mitigation:
   the round-trip probe over 2246 files is the regression test,
   and it runs in under a second
  (Verified:
   `roundtrip-hcl-edit` took 0.255 seconds in the container).
- `hcl-edit` aborts the process at nesting depth 5000 with no knob
  (Verified),
  so until meow's pre-scan exists,
   a hostile or generated file can kill the daemon.
  Mitigation:
   the byte pre-scan described in question 1,
   plus the same limit inside meow's evaluator.
- No error recovery in `hcl-edit`
  (Verified)
  limits the language server's behaviour on a file that is mid-edit.
  This is the most likely reason a future session would move from option 1C to option 1D.
- Writing the evaluator means owning HCL semantics,
  and the 12 Verified divergences in `hcl-rs` are exactly the places a second implementation gets it wrong.
  Mitigation:
   the Go oracle harness in `lab/go-oracle` already answers every case,
  so meow's conformance suite is differential rather than hand written.
- `lsp-types` upstream looks stalled
  (Verified:
   the clone's last commit is 2024-06-04),
  so meow may have to migrate to `ls-types` or `gen-lsp-types` later.
  Mitigation:
   keep protocol types behind meow's own thin layer.
- The corpus is Terraform-shaped.
  Verified:
   it is drawn from `opentofu`,
   `hcl`,
   `hcl-rs`,
   `tofu-ls`,
   and `hcl-lang`,
  plus one synthetic meow inventory.
  A meow-specific corpus does not exist yet,
   so formatter and parser conformance is proven against
  the language,
   not against meow's idioms.

## Tool quirks documented

Three troubleshooting documents were written in the main worktree and left uncommitted,
per the task instruction,
each following the `troubleshooting-doc` skill
(symptom,
 root cause with cited source excerpts,
 verification,
 verified workarounds,
 what does not work,
upstream filing decision against the six constraints,
 duplicate search,
 prototype fix with verification,
and a draft issue in a fenced block):

- `doc/troubleshooting/hcl-edit-binary-operator-decor.md`:
  comments inside a binary operation are dropped on write-back,
  because the operator's `Decor` is never despanned;
  prototype fix adds `self.operator.decor_mut().despan(input);`.
- `doc/troubleshooting/hcl-edit-heredoc-dedent.md`:
  a `<<-` heredoc whose body has a column-zero line loses its `-` and its body indentation on write-back;
  prototype fix returns `Some(0)` from `min_leading_whitespace` in that case and sets the indent to 0.
- `doc/troubleshooting/hcl-rs-eval-arithmetic-and-iteration-order.md`:
  wrapping arithmetic,
   saturation instead of infinity,
   a panic on remainder by zero,
  and insertion-order object iteration;
  prototype fix uses checked arithmetic with a float fallback,
   an explicit divisor-zero error,
  a finite-range guard,
   and a sorted iteration order.

Additional quirks recorded here rather than as their own documents,
because each was an artifact of my own harness rather than a defect a meow user would hit:

- crates.io free-text search returns HTTP 400 beyond page 10,
  so the discovery script caps pagination and the cap is recorded in the vet report.
- `lsp-server` 0.10.0 from crates.io does not build its own integration tests or examples
  (`unresolved import lsp_types`),
  so library tests were run after adding `gen-lsp-types` as a dev dependency to a scratch copy.
- A scripted LSP client hangs forever against all four servers if the client keeps stdin open;
  closing stdin and racing a kill timer fixed it.
- `async-lsp` answers `-32601 No such method shutdown` unless the router registers `request::Shutdown`,
  because `LifecycleLayer` tracks lifecycle state and forwards the request instead of answering it.
- `tree-sitter-grammars/tree-sitter-hcl` publishes 1.1.0 to crates.io while the repository tags v1.2.0,
  and issue 49's infinite loop does not reproduce with the published grammar and `tree-sitter` 0.27.0.

## Vet reports

Two vet reports were written under `doc/audit/` following the `choosing-technology` skill,
and only those two files were committed,
with explicit pathspecs,
 per rule GCE and rule CLG:

- `doc/audit/tech-meow-hcl-front-end-vet-2026-09-17.md`,
   subject "meow HCL front end",
  which recommends `hcl-edit` 0.9.7 as the parse and write-back layer,
   carried patched.
- `doc/audit/tech-meow-language-server-framework-vet-2026-09-17.md`,
  subject "meow language server framework",
  which recommends `lsp-server` 0.10.0.

A tension exists and is recorded in both reports.
Verified (`.claude/skills/choosing-technology/SKILL.md`,
 "Existing tools before custom implementation"):
the skill allows a custom implementation only when every existing tool fails a named hard constraint.
Verified (task statement):
the user's settled requirement is that library gaps are not disqualifiers by themselves.
The reports resolve it by making the hard constraints things that were measured,
namely data loss on write-back,
 process aborts,
 conformance failures against the reference,
and the C-toolchain build constraint,
and by never treating a missing feature as a gate.
The evaluator,
 the function library,
 and the formatter are then meow code by design,
which is what the settled requirement already decided,
and the vetted libraries are the parse layer and the protocol layer underneath that code.

## Process notes

- The two vet reports were written in one pass at the end of the audit
  rather than being created at threshold crossing and updated at each phase,
  which is what the skill's "Update throughout the audit" section asks for.
  Recorded here rather than hidden.
- Rule slips during this work,
  all mine,
   none requested by anyone:
  one shell `for` loop,
   four `;`-chained commands,
   and one foreground `sleep`,
  against rule 1CB and the sleep guidance.
  Later commands used scratch `.ts` scripts instead.

## Questions for the user

Each question below has options,
 pros,
 cons,
 and my ranking.
Every other choice in this document was settled by measurement or by an existing decision,
and is recorded rather than asked,
 per rule QGR.

### Question A: carry a patched `hcl-edit`, or own the parser

Options.

- A1,
   depend on `hcl-edit` and carry the two prototype patches as a vendored crate until upstream fixes land.
  Pros:
  Verified perfect acceptance against the reference on 2246 files;
  spans and comment-preserving tree for free;
  344064 bytes;
  meow writes no lexer.
  Cons:
  a vendored patched dependency to re-apply on every upstream release;
  no error recovery for the language server;
  a process abort at nesting depth 5000 that meow can only avoid by pre-scanning.
- A2,
   own the lexer,
   parser,
   and tree from the start.
  Pros:
  error recovery,
   depth limits,
   exact numbers,
   and a token model the formatter wants;
  nothing third-party can lose a comment on write-back.
  Cons:
  the reference parser and printer are Verified to be 17459 lines of Go plus 20670 lines of tests,
  and meow would be proving a new parser correct before the first configuration evaluates.
- A3,
   depend on unpatched `hcl-edit` and accept the two write-back defects until upstream fixes them.
  Pros:
  no vendoring;
  Verified,
   the defects affect 32 of 2185 corpus files.
  Cons:
  those 32 files lose comments or heredoc markers,
  which contradicts the settled requirement that managed edits preserve comments.

My ranking:
 A1 > A2 > A3.
A1 over A2 because the parser is the one component whose correctness is already Verified against the reference,
and A2 spends the project's first effort re-proving it.
A2 over A3 because A2 is more work but keeps the settled comment guarantee,
while A3 breaks it on measured inputs.

### Question B: impure functions

Options.

- B1,
   no impure functions at all.
  Pros:
  every evaluation is reproducible,
   so the daemon's cache is sound by construction.
  Cons:
  an author who wants a timestamp in a generated file has no way to get one.
- B2,
   provide `timestamp`,
   `uuid`,
   and `bcrypt`,
   and mark any evaluation that calls them as uncacheable.
  Pros:
  matches OpenTofu's name set;
  the cost is contained to the configurations that opt in.
  Cons:
  a single call disables caching for that evaluation,
   which is a cliff an author will hit by accident.
- B3,
   provide them only inside `task` and `check` blocks,
   never in `file` or `mirror` content.
  Pros:
  content that meow writes stays reproducible while ad-hoc tasks stay expressive.
  Cons:
  a function that exists in one block and not another is a rule authors have to learn.

My ranking:
 B1 > B3 > B2.
B1 over B3 because a uniform rule is simpler to explain than a per-block rule,
and no measured need for these functions exists in the incumbent configuration
(Verified:
 `file-enforcer.config.ts` calls none of them).
B3 over B2 because a cache cliff that depends on which function an author happened to call
is harder to reason about than a boundary drawn at a block type.

### Question C: naming for meow-only functions

Options.

- C1,
   plain names alongside the OpenTofu ones:
   `tomldecode`,
   `filetype`,
   `glob`.
  Pros:
  reads the same as the rest of the table;
  short.
  Cons:
  an OpenTofu author cannot tell which names are portable.
- C2,
   a namespace:
   `meow::tomldecode`.
  Pros:
  portability is visible in the source;
  Verified precedent:
   OpenTofu already has namespaced `provider::` functions.
  Cons:
  longer,
   and the namespace appears in every call site.
- C3,
   plain names now,
   a namespace only if a collision with a future OpenTofu function appears.
  Pros:
  short today.
  Cons:
  the rename lands in user configurations later.

My ranking:
 C2 > C1 > C3.
C2 over C1 because the language is deliberately OpenTofu shaped,
so making the non-portable parts visible costs a prefix and prevents a surprise.
C1 over C3 because C3 is C1 with a rename scheduled for the worst possible moment.

### Question D: diagnostic renderer

Options,
with Verified sizes added to a static musl baseline of 455424 bytes.

- D1,
   `codespan-reporting` 0.13,
   57344 bytes.
  Pros:
   smallest;
   plain and predictable output.
  Cons:
   fewer layout features than the others.
- D2,
   `ariadne` 0.6,
   77824 bytes.
  Pros:
   multi-span layouts read well;
   small.
  Cons:
   distinctive style that does not look like `rustc` or `tofu`.
- D3,
   `annotate-snippets` 0.12,
   176128 bytes.
  Pros:
   the renderer `rustc` itself uses,
   so the output looks like the compiler's.
  Cons:
   three times the size of D1.
- D4,
   `miette` 7 with `fancy`,
   307232 bytes.
  Pros:
   richest output,
   with help text and related diagnostics built in.
  Cons:
   largest;
   brings a reporting framework meow would only partly use.
- D5,
   meow writes its own renderer.
  Pros:
   no dependency;
   exact control of wording,
   which rule DGT and rule DNL constrain anyway.
  Cons:
   line wrapping,
   tab handling,
   and Unicode width are the fiddly parts,
   and they are what these crates solve.

My ranking:
 D1 > D3 > D2 > D5 > D4.
D1 over D3 because the extra 118784 bytes buys a `rustc` look that meow has no reason to imitate.
D3 over D2 because a familiar layout beats a distinctive one for a tool people meet through error messages.
D2 over D5 because Unicode width and tab handling are solved there and are tedious to get right.
D5 over D4 because if the output is going to be meow's own wording anyway,
a self-written renderer is smaller than the largest dependency measured.
This question is asked rather than settled because no measurement distinguishes the options
once size is accepted as small on all of them,
and because the choice is about how meow's output should look.

### Question E: upstream filing, once the freeze lifts

The task instruction was explicit:
 no upstream filing or commenting during this work,
and that was followed.
Three defects now have root causes,
 prototype fixes,
 and draft issues ready in the troubleshooting documents.

Options.

- E1,
   file all three upstream later,
   with the prototype patches as pull requests.
  Pros:
  meow eventually drops its vendored patches;
  the fixes help every other `hcl-rs` user.
  Cons:
  review latency is outside meow's control,
   and the vendored patch has to exist meanwhile anyway.
- E2,
   file only the two `hcl-edit` write-back defects,
   and keep the evaluator findings private
  because meow will not use `hcl-rs`'s evaluator.
  Pros:
  narrower,
   and the two are the ones meow actually depends on.
  Cons:
  the evaluator findings are the most detailed of the three and would help the next person.
- E3,
   file nothing.
  Pros:
  no ongoing upstream relationship to maintain.
  Cons:
  meow carries patches forever.

My ranking:
 E1 > E2 > E3.
E1 over E2 because the evaluator report costs nothing extra to file now that it is written,
and filing it does not oblige meow to use the evaluator.
E2 over E3 because the two write-back defects are the ones meow's vendored patch exists for,
so upstreaming them is the only way the patch ever goes away.

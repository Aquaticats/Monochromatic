# git-policy-cli-fuzz

Coverage-guided checks for the native monochromatic-git-policy-cli wrapper's pure boundaries.
The subject is `package/git-policy/cli/src/native`;
nothing here starts Git or touches a repository.

## Targets

### `global_arguments`

Runs `global_layout` over two views of every input:
the bytes split at NUL into arguments,
and the bytes mapped to real Git option,
command,
and boundary tokens.
It checks that the arguments are unchanged,
that the result is repeatable,
that the boundary token has the kind the outcome claims,
that the global prefix alone names no command,
and that nothing after a decided boundary changes the result.

### `config_loading`

Runs `classify_config_loading` over the same two views.
Skipping configuration skips policy,
so it checks that only native queries,
option errors,
and an independently restated list of inspection commands skip,
that an absent or unknown command never skips,
and that any `branch` or `tag` invocation gaining a mutating flag in first position requires configuration.

### `config_schema`

Builds a valid `monochromatic-git-policy-cli.config.jsonc` document from every input together with the settings it must parse to,
and compares the schema's result with that expectation.
For `security/forbidden-strings` the generator also writes `rulesFile` as a string or as an array of strings and
`{"path", "optional"}` objects,
with paths that hold a comma or a question mark (a configured path is never parsed as a list),
and the canonical restatement writes every file as an object with its `optional` flag.
Inputs that are UTF-8 are also parsed as written.
An accepted document must satisfy the registry-order and bound invariants
and survive being restated canonically.
A rejection must render as one `config-invalid` JSON line that decodes back to the same message.

### `wrapper_controls`

Runs wrapper-control removal and the wrapped-command lifecycle over four views of every input:
the bytes split at NUL into arguments,
the bytes mapped to commands,
Git options,
separators,
value options,
every wrapper control and hatch,
and near-misses of their spellings,
a `<command> <valueless options> -- <tokens>` list whose `--` is certainly Git's separator,
and the same tokens behind a first byte that selects a linked worktree or no repository
and the answers to the repository questions.
Removal must leave its input unchanged and be repeatable,
delete only whole tokens that spell a control or the commit hatch,
record exactly the effects the deleted tokens asked for,
find nothing more on a second pass,
and keep everything from the separator on.
The lifecycle runs with fixed repository answers and locations that do not exist on disk,
so it reads no configuration and starts no Git.
Whatever the arguments,
a forwarded command must not be a `git commit` without a dry-run or status-format option,
a `git push` without a dry-run option while the built-in content policy is on,
or a `git add` in a worktree while that policy is on
(the fixed answers prepare no candidates,
so that policy cannot read them).
A worktree creation or possible alias run from a linked worktree is forwarded:
the executable settles the worktree copy around it.
The dry-run options are restated from Git's documentation instead of read from the subject's tables.

### `batch_reply`

Runs `read_batch_reply`,
the reader of one `git cat-file --batch` reply,
over two views of every input:
the bytes split at the first line feed into a request and a raw stream,
and the bytes mapped to a built reply together with the outcome it must have
(a canonical reply,
a missing notice,
a reply for another object,
content shorter or longer than declared,
a stream that ends early,
an overlong header,
a header Git never prints,
or a reply above the subject's `CONTENT_CEILING`,
alone,
followed by further bytes,
cut short,
without its terminator,
or for another object).
Reply content is file content,
so it may imitate a reply.
It checks that reading is repeatable,
that an accepted reply is byte for byte Git's canonical rendering of the returned value,
that a reply to a request by name names that object,
that bytes after the reply change nothing,
that no sampled proper prefix of an accepted reply is accepted,
that a skipped reply holds exactly its header and the bytes it declares,
that only a reply above the ceiling is skipped,
and that a refusal is one of the four reply failures.
The generator shortens the ceiling instead of staging it,
because the reader takes the ceiling as an argument.
The target reads in-memory bytes only;
the process that produces real replies is controlled beside the subject.

### `stage_listing`

Runs `parse_stage_records`,
the reader of `git ls-files --stage -z` output that a `git add` prediction lists before and after the add,
over the raw bytes,
and `staged_delta` over two listings built from them in Git's index order,
with conflict stages,
every mode,
and pathnames holding tabs,
spaces and bytes that are not UTF-8.
An accepted listing must be byte for byte Git's rendering of the returned records,
and a refusal must be a malformed listing or an unsupported mode.
The delta must equal one recomputed from sorted maps:
every pathname whose records differ,
with its records afterwards,
pathnames of the first listing first and new ones after them.

### `rules_file`

Runs `check_rules_file`,
the check of the `rulesFile` option of `security/forbidden-strings`,
over text inputs as written and over values built from path words
(separators,
`.`,
`..`,
drive prefixes,
backslashes and NUL).
The answer must be the refusal restated from the value's words in the documented order,
and an accepted value joined to a root must stay below it,
with only ordinary components.

### `final_newline`

Runs `normalized_final_newline`,
the `final-newline` rule that also decides what `git monochromatic-git-policy-cli fix` writes,
over raw bytes and over text built with a chosen number of trailing line feeds.
A file is left alone only when it is empty,
holds NUL,
is not UTF-8,
or already ends with exactly one LF;
a replacement keeps every byte before the trailing LF run,
ends with exactly one LF,
and is itself left alone.

### `owner_records`

Runs the copy owner token reader,
the readers of the owner token in a journal name and a temporary file name,
the transaction owner record reader
and the `/proc/<pid>/stat` start-time reader
over raw bytes and over tokens,
names,
records and stat lines built from the input.
A copy's journal,
stage claim and temporary file name their owner in the file name
(`package/git-policy/cli/src/native/worktree_copying/worktree_copy_owner.rs`),
which replaced the owner lock record that commit `ee30d244f6` removed with the settlement lock.
A built owner is restated by the subject's token encoder
and then edited once by one of a fixed list of field edits
(a PID of 0,
negative,
fractional,
beyond the integer range,
empty or text,
an empty birth identity,
an escape cut short,
not hexadecimal,
or not UTF-8,
a fourth field,
a missing namespace field,
and a plain letter respelled as its equal escape),
and the reader must return exactly the outcome the edit implies.
The built token must hold only characters a part of a file name can carry,
and the journal and temporary names built around it must give it back.
Any accepted token must have a positive PID and a non-empty birth identity,
and must parse back to itself after being restated.
Any text read as a journal or temporary name must end in its suffix and give a non-empty token without a dot;
a journal is read through its path's last component,
so a text ending with a separator names the component before it.
A transaction owner record is parsed as written
and restated;
an accepted one must have a positive safe PID and a non-empty identity.
A built stat line carries a command name holding spaces,
parentheses and `) ` sequences,
a process state,
and the start time in field 22;
the reader must return that start time for a running process,
nothing for a zombie or dead one,
and refuse a line without the start time or without a closing parenthesis.

### `owner_claim_record`

Runs `decode`,
the reader of a single-owner claim file
(`package/git-policy/cli/src/native/lock/single_owner_claim.rs`),
over raw text and over claim files built from the input.
A built file chooses its first line
(the current folder spelling,
the earlier one,
the current one in capitals,
or the header with no folder name)
and then one of six shapes:
a released claim,
the same followed by a line no reader acts on,
a whole owned claim,
the same followed by such a line,
an owned claim one field edit leaves naming no process
(a process id that names none,
an empty birth identity,
or no namespace line),
or a released line a trailing space makes a near miss.
Its process id is a positive one,
zero,
a negative one,
a fraction,
a value past the integer range,
a leading-zero spelling,
or text;
its birth identity is text holding quotes,
escapes,
a NUL,
a space,
the released marker and non-ASCII characters,
or empty;
its namespace is the released marker,
empty,
that text,
or the line is left out.
An accepted claim must name one of the two folder spellings,
a released claim or a process id of at least one with a non-empty birth identity,
and a namespace that is never the released marker,
and it must hold exactly its own record when restated here in the current spelling.
A hand-written file whose field ends in a carriage return is read without that carriage return,
because line splitting removes it,
so that record is not restated;
raw bytes still reach the decoder for the refusal rules and for panics.

### `dependent_version`

Runs the dependent-version planner's two text scanners over three views of every input.
The first builds a manifest from the input bytes:
members before and after one top-level `version` key,
chosen from look-alikes
(a nested `version` key,
`"version"` as a value,
escaped quotes,
slashes and closing brackets inside strings),
in a byte-chosen layout,
together with the text the rewrite must produce,
assembled from the same parts with the new version.
The rewrite must produce exactly that text.
The second splits the input at its first two NUL bytes into `from`,
`to`,
and raw manifest text.
A successful rewrite must replace exactly one occurrence of the quoted `from` with the quoted `to`,
and rewriting back must restore the input;
a refusal must be a manifest shape problem;
and when the manifest reader accepts the text and it holds exactly one plainly spelled
top-level version equal to `from`,
the rewrite must succeed and the reader must then see `to` with every other fact unchanged.
The third splits the input at its first NUL byte into a package name and source text,
and requires the specifier scan to agree with a restatement of its rule written with index loops,
including for an empty name.
The controls rewrite every generated layout of every single-byte fill,
every sequence of up to four manifest tokens,
and scan every sequence of up to four specifier tokens for three names.
The planted controls add three defects:
a nested `version` key rewritten,
a subpath import missed,
and an escaped quote that ends a string literal
(`bin/planted-dependent-version.ts`).

### `linter_output`

Runs `interpret_embedded_run`,
the reader of one in-process `monochromatic-lint` fix run that the Markdown policy trusts,
over two views of every input:
the bytes mapped to how the run ended
(a usable run,
or a failure with status 1,
status 2 or another status),
a fixed source and findings,
and the bytes mapped to findings with one planted corruption
(another rule,
no code,
an error severity,
no label,
a code without the `markdown/` prefix,
or a code with a suffix).
Only a usable run whose fixed source is not empty for a non-empty file,
and whose findings are all warnings of the selected rule with a position,
may be used,
and then exactly as stated,
with the position of the first label;
every other run must be refused with a reason that names the status,
the finding or the empty source.
The two size bounds are far above the input limit,
so a control builds the two oversized runs instead.

## Controls

`mise run //package/git-policy/cli.fuzz:test` runs the generator controls.
They count that the generators reach every layout outcome,
both loading decisions,
non-default accepted configurations,
every way a command region is read,
every control effect,
a control spelling that survives as a value or path,
every ending of the lifecycle,
every reply kind and reply failure,
equal,
changed,
removed and new paths of a staged delta,
every `rulesFile` refusal,
every final-newline outcome,
accepted and refused owner tokens,
every claim outcome
(a released claim,
an owned claim and a refusal),
running,
exited and malformed stat lines,
so an invariant that is never reached cannot pass unnoticed.

`mise run //package/git-policy/cli.fuzz:test:planted` proves the invariants can fail.
It copies the subject and this package to a temporary directory,
plants one defect at a time
(a mutating `branch` letter accepted as presentation,
a bare `git` skipping configuration,
an unconsumed global option value,
`warn` read as `error`,
a rejected `landing` section,
a control spelling removed anywhere in a region read without a table,
keep-going removed without being recorded,
a forwarded real commit,
a publishing push that skips the manual-push gate,
object content accepted past its declared size,
a reply accepted for another object,
a changed path left out of the staged delta,
a `rulesFile` value with `..` accepted,
an owner token naming PID 0 accepted,
a claim naming PID 0 accepted,
the start time read from the field before it,
extra final line feeds kept),
and requires a generator control to fail for each.
Results are retained under `target/verification/planted-*`.
Removing a flag from a mutation list alone is not a usable plant:
unlisted `branch` and `tag` flags already require configuration.

The `fuzz_target!` input looks like a closure signature but is macro input grammar.

## The lockfile

`Cargo.lock` here must lock every package the subject locks,
at the same version.
A sidecar links the subject by path,
so it resolves the subject's whole dependency graph,
and a dependency the subject gains,
drops or moves makes this lock stale.
The subject's `lint:sidecar-locks`
(`package/git-policy/cli/bin/native-sidecar-lock-check.ts`)
fails on a lock that fell behind,
and the subject's container gate runs the same check before it builds anything;
the root `lint` still reaches this package's own `lint`,
which compiles the targets against the subject.
`mise run //package/git-policy/cli.fuzz:lock:members` records the versions again,
and `lock:subject` seeds this lock from the subject's,
so both lock the same registry versions.

`.github/workflows/git-policy-cli-fuzz.yml` compiles this package against the subject
on every pull request and push that touches either one,
by running `lint:types` through the task CLI CI uses
(`package/git-policy/cli.fuzz/task.ts` is its mise-free definition,
`doc/decision/mise-free-ci.md`).
So a stale lock and a subject change that moved a public type
both fail CI before a campaign would have hit them;
the failure was measured in `doc/planning/catching-a-stale-fuzz-sidecar-lockfile.md`,
and the local path stays the root `lint` and this package's `lint`.

## Campaigns

`mise run //package/git-policy/cli.fuzz:smoke` builds AddressSanitizer targets with the read-only nightly compiler,
runs the controls and Clippy in the same bounded container,
then fuzzes each target for 30 seconds without host mounts
under the shared container bounds
(`defaultContainerLimits`,
re-exported by `package/git-policy/cli/bin/container-limits.ts`:
2 GiB,
2 CPUs,
512 PIDs),
and a 4,096-byte input limit
(`maxInputBytes` in `bin/fuzz-images.ts`).
A target must exit 0 and report at least one executed unit.
A longer campaign passes the time per target after the task name,
for example `mise run //package/git-policy/cli.fuzz:smoke -- --seconds-per-target=1800`;
the value is a whole number of seconds from 1 to 86,400,
and the manifest in the evidence directory records it.
`--targets=<name,...>` limits the run to the named targets.
Each target's standard output and standard error go straight to `stdout.log` and `stderr.log` in the evidence directory,
because a target such as `wrapper_controls` prints a line for every commit it forwards
(about 140 KiB per second measured on 2026-10-17),
which overflowed the earlier in-memory capture.
Corpora,
artifacts,
and logs are retained under `target/verification/campaign-*`.

Retain and replay minimized failures,
and convert them into deterministic tests beside the subject.
These targets do not cover real-Git resolution,
forwarding,
configuration file reading,
repository facts read from Git,
transaction recovery on disk,
or the management grammar.

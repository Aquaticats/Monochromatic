# git-policy-cli-fuzz

Coverage-guided checks for the native cli-git wrapper's pure boundaries.
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

Builds a valid `cli-git.config.jsonc` document from every input together with the settings it must parse to,
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
(the fixed answers prepare no candidates, so that policy cannot read them).
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
or a header Git never prints).
Reply content is file content,
so it may imitate a reply.
It checks that reading is repeatable,
that an accepted reply is byte for byte Git's canonical rendering of the returned value,
that a reply to a request by name names that object,
that bytes after the reply change nothing,
that no sampled proper prefix of an accepted reply is accepted,
and that a refusal is one of the four reply failures.
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
the `final-newline` rule that also decides what `git cli-git fix` writes,
over raw bytes and over text built with a chosen number of trailing line feeds.
A file is left alone only when it is empty,
holds NUL,
is not UTF-8,
or already ends with exactly one LF;
a replacement keeps every byte before the trailing LF run,
ends with exactly one LF,
and is itself left alone.

### `owner_records`

Runs the owner lock record reader,
the transaction owner record reader
and the `/proc/<pid>/stat` start-time reader
over raw bytes and over records and stat lines built from the input.
A built owner lock record is restated by the subject's encoder
and then edited once by one of a fixed list of field edits
(a PID of 0, negative, fractional, beyond the safe range or text,
an empty or renamed token,
an empty identity,
another schema version,
an equal schema version spelled `1.0`,
a mistyped `transactionId` before the real one,
an unknown field),
and the reader must return exactly the outcome the edit implies,
with the last of duplicate keys winning as in `JSON.parse`.
A built stat line carries a command name holding spaces,
parentheses and `) ` sequences,
a process state,
and the start time in field 22;
the reader must return that start time for a running process,
nothing for a zombie or dead one,
and refuse a line without the start time or without a closing parenthesis.
Any accepted record must have a positive safe PID and non-empty token and identity,
and must parse back to itself after being restated.

### `transaction_records`

Runs the journal readers
(`preparing.json`, `landing-<n>.json`, `index-lock-<n>.json`, `ref-updated.json`),
the capture-order readers (`captured.json`, landed-capture records)
and the capture sequence reader
over raw bytes and over records built from the input through the subject's types.
Every built record must parse back to itself;
one edit per input
(another schema version, a schema version spelled as text, another state,
an array around the record, a mistyped `fsId`, an attempt of 0)
must make every record kind it applies to refuse,
and a proper prefix of a landing record must be refused.
An accepted record restated by the encoder must parse to the same record,
a landing record is a commit exactly when it names its new commit,
and the sequence reader accepts exactly a canonical decimal and a newline.
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
equal, changed, removed and new paths of a staged delta,
every `rulesFile` refusal,
every final-newline outcome,
accepted and refused owner lock records,
running, exited and malformed stat lines,
and normalizations, detached heads and merge conclusions in journal records,
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
an owner lock naming PID 0 accepted,
the start time read from the field before it,
a journal record of another schema version accepted,
a capture record with sequence 0 accepted,
a sequence file with a leading zero accepted,
extra final line feeds kept),
and requires a generator control to fail for each.
Results are retained under `target/verification/planted-*`.
Removing a flag from a mutation list alone is not a usable plant:
unlisted `branch` and `tag` flags already require configuration.

The `fuzz_target!` input looks like a closure signature but is macro input grammar.

## Campaigns

`mise run //package/git-policy/cli.fuzz:smoke` builds AddressSanitizer targets with the read-only nightly compiler,
runs the controls and Clippy in the same bounded container,
then fuzzes each target for 30 seconds without host mounts
under 2 GiB,
2 CPUs,
128 PIDs,
and a 4,096-byte input limit.
A target must exit 0 and report at least one executed unit.
A longer campaign passes the time per target after the task name,
for example `mise run //package/git-policy/cli.fuzz:smoke -- --seconds-per-target=1800`;
the value is a whole number of seconds from 1 to 86,400,
and the manifest in the evidence directory records it.
`--targets=<name,...>` limits the run to the named targets.
Each target's standard output and standard error go straight to `stdout.log` and `stderr.log` in the evidence directory,
because a target such as `wrapper_controls` prints a line for every commit it forwards
(about 140 KiB per second measured on 2026-10-17), which overflowed the earlier in-memory capture.
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

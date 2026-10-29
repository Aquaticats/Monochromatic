# @monochromatic-dev/mutation-test-cli

Container-native mutation testing framework built on yuku.

Replaces the StrykerJS-based `dev-script-mutation-test`:
mutants are enumerated host-side with `yuku-parser` and applied by span splicing,
then executed in sharded disposable Podman containers with taint-aware re-runs.

## Trust model

A mutant is arbitrary bad code.
Once the first mutant executes in a container,
 that container is untrusted:

- Mutants are grouped into shards,
   one disposable container per shard,
  bounding how many results share a tainted container.
- Any anomaly (per-mutant timeout,
   runtime error,
   restore failure,
   container failure)
  re-runs the shard remainder in fresh half-size shards;
   bisection bottoms out at
  single-mutant shards,
   so every mutant eventually gets an untainted position-1 run
  if needed.
- Every Survived and final Timeout result is confirmed as the first mutant in a
  fresh container before being reported.
   Killed results are accepted from any
  shard position.

## Statuses and reporting

Native versioned JSON report;
 no mutation score.
Statuses:
 killed,
 survived,
 timeout,
 compileError,
 runtimeError.
Each mutant record carries provenance:
 shard id,
 position,
 rerun count,
 confirmed flag.
Exit code is zero when the run completes (survivors included);
 nonzero only on
infrastructure failure.

## Work-tree copy

Each shard container runs in a tmpfs work tree (`--work-tmpfs-size`, default 2g),
so the container must not copy the whole checkout:
ignored build output and nested agent worktrees can be gigabytes.
The host runs `git ls-files -z --cached --others --exclude-standard` once per run
(tracked plus untracked-not-ignored, so uncommitted edits are still tested),
writes the NUL-separated list next to each shard manifest,
and the container copies exactly those paths with `rsync --files-from`.
A tracked file deleted in the working tree is skipped,
and generated directories such as `node_modules` and `dist` stay excluded as a second guard.

## Type-check filter

Each mutant is checked with `tsgo --noEmit --incremental --project` inside the
shard container (warm `.tsbuildinfo` from the baseline check);
 non-compiling
mutants are reported as compileError and skip test execution.

## Usage

```bash
# Full run against one package (report JSON lands beside the cwd by default)
mutation-test --package package/fs-path/core

# Enumerate mutants and selected tests without any containers
mutation-test --package package/fs-path/core --dry-run

# Narrow to specific source files (positional, package-relative)
mutation-test --package package/fs-path/core src/trim.ts
```

Flags:
 `--full-suite` (every unit test instead of stem-related selection),
`--shard-size` (default 16),
 `--containers` (concurrent shard containers,
default 2),
 `--memory`/`--cpus`/`--pids-limit`/`--work-tmpfs-size`/
`--session-timeout-seconds` (per-container caps),
 `--timeout-ms` (per-mutant
floor) and `--timeout-factor` (multiple of baseline test time),
`--selinux-relabel`,
 `--skip-image-build`,
 `--report <file>`.

Test selection:
 `<stem>.unit.test.ts` plus dot-sidecars
(`<stem>.regression.unit.test.ts`) plus package-level
`src/integration.unit.test.ts` when present.
 Files no test selects skip
containers entirely;
 their mutants report as confirmed survivors.

## Suppression

```ts
// mutation-test-disable-next-line string, boolean -- filler noise
export const label = 'exact copy matters here';
```

`mutation-test-disable-next-line [families] [-- reason]` suppresses the next
line;
 `mutation-test-disable-file [families] [-- reason]` suppresses the whole
file.
 Bare directives suppress every family;
 unknown family names throw at
enumeration.
 Suppressed mutants land in the report's `ignored` bucket with
their reasons.

### Suppressing one replacement

A family name followed by a colon and a replacement hides only the mutants
that replace the code with exactly that text,
 so an equivalent mutant can be suppressed without hiding its siblings:

```ts
// mutation-test-disable-next-line equality:>= -- the tie cannot happen
if (index > stopIndex) return;
// mutation-test-disable-next-line conditional:false, block -- the guard only skips work
if (!isNeeded) return;
```

The text is compared to the mutant's `replacement` field in the report,
 character for character.
 A bare family name keeps meaning every replacement of that family,
 and a bare mention wins over restricted mentions of the same family.
 Listing the same family with several replacements hides each of them,
 and repeating an entry changes nothing.

A replacement is written bare,
 as a run of characters without whitespace,
 commas,
 or double quotes (`equality:>=`,
 `update:--`),
 or as a JSON string literal that can hold any text:
 `string:"a, b: c -- d"` holds commas,
 a colon,
 spaces,
 and the reason separator,
 `method:"x.slice(\n  0,\n  1,\n)"` holds newlines,
 and `optional-chaining:""` is the empty string.
 These are rejected at enumeration with a message that says what to change:
 a colon with nothing after it
 (write `""` for the empty string),
 a bare replacement with a double quote,
 an unterminated or non-JSON quoted literal,
 text after a replacement that is neither a comma nor the reason separator
 (for example a bare replacement with a space),
 and an empty or trailing family entry.

Families:
 arithmetic,
 equality,
 logical,
 conditional,
 boolean,
 string,
 unary,
update,
 array,
 object,
 optional-chaining,
 block,
 method,
 arrow,
 regex.
The regex family is a reduced-scope token mutator (quantifier swaps,
 anchor
drops,
 escape-class negations),
 not a weapon-regex port.

## Design record

<https://github.com/Aquaticat/Monochromatic/issues/247#issuecomment-4887670850>
(one correction:
 parser JS bindings,
 oxc-parser then and yuku-parser now,
return UTF-16 string offsets,
 so the
splicer works on JS string slices,
 not Buffers).
Integration expectations live in `package/mutation-test/cli.fixture`.

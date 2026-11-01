# sccache v0.18.0 records a failed compiler request during successful Cargo builds

## Symptom

In standalone Depot CI
[run `gmrm0ff71h`](https://depot.dev/orgs/l4zncjjbj5/workflows/gmrm0ff71h),
the seed,
unchanged consumer,
and changed-flag jobs all completed successfully.
Each `cargo build` returned zero,
and each independent executable passed its public library checks.
Every sccache statistics object nevertheless contained `compile_fails=1`.
Each also recorded 20 compiler requests and eight non-cacheable requests.
The [full receipt](../audit/depot-sccache-fresh-jobs-2026-10-18.evidence.json)
preserves the counters and command outcomes.

## Root cause boundary

The exact compiler subprocess behind the counter was not traced.
No specific Cargo or build-script probe is established as its cause.
The counter does not identify which command failed or whether Cargo expected that result.

In sccache release `v0.18.0`,
commit `01c35e69b97f516c2ebfbdb418b5cd6277bde280`,
`src/server.rs:1563` increments the counter for a failed compilation result:

```rust
//! Excerpt from mozilla/sccache/src/server.rs.
CompileResult::CompileFailed(dt, duration) => {
    debug!("[{}]: compile result: compile failed", out_pretty);
    dist_type = dt;
    stats.compilations += 1;
    stats.compiler_write_duration += duration;
    stats.compile_fails += 1;
}
```

`src/server.rs:1600` also increments it when request handling returns a compiler-process error:

```rust
//! Excerpt from mozilla/sccache/src/server.rs.
Ok(ProcessError(output)) => {
    debug!("Compilation failed: {:?}", output);
    stats.compile_fails += 1;
```

These source branches explain the counter's scope,
not which branch or subprocess produced this observation.
The source was cloned and read without modification.

## Verification

The pushed harness is branch `trial/depot-cache-disk-20261018T0503`,
commit `298860fd9db1b894bba0f4531f2307df6881e0b0`.
Its source bytes and hashes are retained in the receipt.
It uses the upstream x86_64-musl release archive,
SHA256 `45f1447fbe231e3037bde351ef70677dd212216c8d62ae7ca409fecc4d6acc89`,
and verifies executable SHA256
`973cb15f6a986d84ca334bbed3bbe2eb8f1ee8fd81bf9e115b8539a293bf8d59`.
The actual compiler was rustc `1.97.1`.

To reproduce in a disposable checkout,
change the unique cache prefix,
commit and push the harness,
then run:

```sh
# Disposable checkout containing the committed compiler-cache harness
depot ci run --workflow .depot/workflows/depot-sccache-probe.yml --org l4zncjjbj5 --repo Aquaticats/Monochromatic
```

This dispatch is bounded to three fresh two-CPU/eight-GB allocations,
each with a five-minute timeout.
It builds the existing `truepeak-core` default-feature library with its lockfile.
It does not restore a Cargo target or registry archive.

### Passing observations

- The seed wrote 11 Rust cache entries;
  the unchanged fresh allocation recorded 11 Rust hits and no Rust misses.
- Changed flags produced 11 Rust misses and successful writes.
- Cache read/write error counters were zero.
- Cargo builds,
  independent public gain/meter consumers,
  and sccache server shutdown returned success.
- The seed and unchanged library archive hashes matched.

### Non-success and unexercised observations

- The compiler-request failure counter was one in every job.
- `cargo nextest --version` returned 101 with `` error: no such command: `nextest` ``.
  No unit-test execution or pass was claimed.
- Optional features,
  application builds,
  and the failed subprocess's identity were not exercised or diagnosed.

## Verified harness handling

The executed harness preserved all statistics and command stderr,
checked outer command exit status,
checked real consumer behavior,
and asserted the cache's hit/miss and error boundaries independently.
This passed in every allocation.
Its tradeoff is that success proves those explicit boundaries,
not that every compiler subprocess succeeded or that all library tests ran.
This is an experiment result rather than an adopted production gating policy.

## What does not work

Treating `compile_fails=1` as proof that the Cargo build failed contradicts its recorded exit status.
Treating a successful cache hit as proof that every compiler request succeeded contradicts the retained counter.
Treating successful builds and consumer checks as a unit-test pass contradicts the missing Nextest command.

## Upstream filing decision

The `.out-of-scope/` entries were searched for sccache,
Rust,
and probe-related exemptions;
no matching sccache exemption was found.
Issue searches for `compilation failures cargo`
and pull-request searches for `compile_fails` did not establish a matching diagnosed incident.
Broad counter matches alone were not treated as the same failure.

- An upstream fault is not established:
  a request-failure count and an enclosing build exit status describe different boundaries.
- A fix cannot be scoped before identifying the failed request and whether its failure was expected.
- Rust compiler caching is supported,
  and the real remote-cache controls passed.
- Contribution eligibility was not assessed for a proposed change,
  because no defect or patch was identified.
- Upstream acceptance or rejection was not inferred from the observation.
- No patch was prototyped:
  the prerequisite of an established upstream fault failed.

Nothing is fileable from this evidence.
No upstream message was sent.

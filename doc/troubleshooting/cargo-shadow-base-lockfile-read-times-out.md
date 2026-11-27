# cargo-shadow refuses a build when its read of the base lockfile times out

`cargo` in this repository is cargo-shadow.
Before it runs the command it was given,
it compares the package's `Cargo.lock` with the lockfile at its verification base,
and it verifies only the locked versions that are new since that base.
One case widens that check to every version in the lockfile:
a read of the base's lockfile that did not finish inside its bound.

## Symptom

A round's first `cargo test` in a worktree refused to run the tests and printed:

```text
cargo-shadow: comparing lockfiles with origin/main (61274c4d32e110b92e1f5e4a16b3ab23b72e262a)
cargo-shadow: warning: `git show 61274c4d32e110b92e1f5e4a16b3ab23b72e262a:package/git-policy/cli/Cargo.lock` (cargo-shadow reads the lockfile of the verification base) did not finish within 60s; it and every process it started were stopped. It may have changed files before it was stopped; this tool did not restore or remove anything because of that. Run `cargo test --locked --offline --lib --features test-only-switches kept_references:: -- --skip guest --skip _arm_tests` again when it can finish; the lockfile is treated as absent at the base, so every version in it is checked
cargo-shadow: verifying 323 versions newly locked since the base from 1 lockfiles
cargo-shadow: checking the trust policy for 323 versions of 314 crates
Install refused; no Cargo.lock was left changed: 2 finding(s).
- cpufeatures@0.3.1 has trust level 2, below an earlier release's 3 ...
- rowan@0.15.18 has trust level 1, below an earlier release's 2 ...
```

Nothing about the worktree had changed:
its lockfile was `main`'s own,
and the two flagged crates had been locked there for days.

## Cause

`package/cargo-shadow/cli/src/git_base.rs` reads the base lockfile with a bounded `git show`,
and the bound is `GIT_CALL_SECONDS`:
60 seconds,
from `package/cargo-shadow/cli/src/git_refs_environment.rs`.
On a timeout it prints the warning above and answers with an empty lockfile text,
so every locked version counts as one that no lockfile held at the base.
That direction fails safe for the trust policy,
checking more versions rather than fewer,
but a trust finding about an old release then refuses a command whose own package never changed.

The bound is reachable on a busy host:
on this one the same `git show` took 21.4 seconds when measured by hand,
with 61 GB of memory and 10 CPUs under about twenty concurrent agent builds.
That measurement was `time git show 61274c4d32e110b92e1f5e4a16b3ab23b72e262a:package/git-policy/cli/Cargo.lock | wc -l`.

## What a round does

Run the command again once the base read finishes inside its bound.
The second attempt printed `verifying 0 versions newly locked since the base`,
ran the tests,
and passed.

Do not add a `trustPolicyExclude` line for the flagged crates on a round's own judgement:
`rust-catalog.toml` records each exclusion as a reviewed release,
and reviewing `cpufeatures@0.3.1` and `rowan@0.15.18` is the owner's decision
(`doc/decision/rust-dependency-catalog.md`,
section "Update and tightening").

## What would end this

Either cargo-shadow reads the base lockfile without a bound a busy host can reach,
or a timeout there is reported as an unknown base that refuses the run,
not as one that widens the trust check,
so a busy host fails the command with its real cause rather than with two old releases.

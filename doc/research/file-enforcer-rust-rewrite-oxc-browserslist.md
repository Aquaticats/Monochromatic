# oxc-browserslist against browserslist-rs and npm browserslist

## Purpose and outcome

The handover set the next action after the 2026-10-29 reboot:
look at `oxc-browserslist` as an alternative to `browserslist-rs`,
whose data the gate probe measured behind npm `browserslist`.
This report records what that look found on 2026-10-30.
Partway through,
the user replaced both crates with an in-repo implementation
("Change of plans,
for browserslist,
we retrieve the data and write something conforming to the original browserslist ourselves,
only supporting the queries we use now,
throw on everything non-conforming with \"not implemented\".");
the planned final arm,
regenerating oxc's data from the repository's locked npm packages,
was dropped.
The findings stay here as knowledge
(the user,
2026-10-29:
"Even if work is superseded,
it should still become knowledge.").

## Real-config probe

Inputs:
the repository's `.browserslistrc` (seven default lines,
no sections),
copied to `${HOME}/temp/agent/fe-rust/oxc/w/repo-copy/`.
Each Rust arm calls `resolve` with the seven lines,
then each line alone,
then `ie 99999` as a positive control that must fail.
Harness:
`${HOME}/temp/agent/fe-rust/oxc/w/{rel,main}/src/main.rs`,
built and run in `podman run --rm --memory=4g --cpus=4 docker.io/library/rust:1`
(rustc 1.99.1),
with the oxc clone mounted read-only.
The npm arm is the gate probe's (`doc/research/file-enforcer-rust-rewrite-gate-probes.md`, "Probe 2");
the repository's current `.browserslistrc.resolved.local.json` is byte-equal to it after `jq --compact-output`.

Results for the seven lines together:

- npm `browserslist` 4.29.5 (`caniuse-lite` 1.0.30001819,
  `node-releases` 2.0.60):
  `and_chr 156`,
  `and_ff 158`,
  `android 156`,
  `chrome 156`,
  `firefox 153`,
  `node 26.12.0` down to `node 26.0.0`;
  18 entries.
- `browserslist-rs` 0.21.2 (gate probe):
  `and_chr 151`,
  `and_ff 153`,
  `android 151`,
  `chrome 151`,
  `firefox 140`,
  `node 26.8.0` down to `node 26.0.0`;
  14 entries.
- `oxc-browserslist` 5.0.1 from crates.io:
  `and_chr 151`,
  `and_ff 153`,
  `android 151`,
  `chrome 151`,
  `firefox 140`,
  `node 26.7.0` down to `node 26.0.0`;
  13 entries.
- `oxc-browserslist` main at `6297028` (2026-10-23,
  unreleased):
  `and_chr 154`,
  `and_ff 156`,
  `android 154`,
  `chrome 154`,
  `firefox 140`,
  `node 26.10.0` down to `node 26.0.0`;
  16 entries.

Query handling matched npm in every Rust arm:
the same families,
output shape,
and order;
a standalone `not dead` fails
("query cannot start with 'not'; add any other queries before 'not dead'"),
and the control fails ("unknown version '99999' of browser 'ie'"),
so the harness can show errors.

## Firefox ESR

Every Rust arm returns `firefox 140`.
`oxc-browserslist` hard-codes it in `src/queries/firefox_esr.rs:3-4`:

```rust
// src/queries/firefox_esr.rs (oxc-browserslist main 6297028)
pub(super) fn firefox_esr() -> QueryResult {
    Ok(vec![Distrib::new("firefox", "140")])
}
```

`browserslist-rs` 0.21.2 hard-codes the same value (`src/queries/firefox_esr.rs:4`, gate probe).
npm `browserslist` 4.29.5 hard-codes `var FIREFOX_ESR_VERSION = '153'` (`index.js:15`).
So the ESR answer is a constant in every implementation,
and refreshing data never changes it.

## Release cadence and data freshness

From the crates.io versions API (fetched 2026-10-30):

- `oxc-browserslist` released often until 5.0.1 on 2026-08-12
  (for example 3.0.4 to 3.0.13 between 2026-05-29 and 2026-08-03),
  then nothing.
- `browserslist-rs` released 0.20.0 on 2026-08-01 and 0.21.2 on 2026-09-16.

oxc's `.github/workflows/cron.yml` runs daily:
`vp update --prod`,
`cargo codegen`,
`cargo test`,
then opens and auto-merges an "Update browserslist" pull request,
so `src/generated` on main last changed 2026-10-01 (`3d95fdc`),
with `caniuse-lite` 1.0.30001813 in its `pnpm-lock.yaml`.
Releases go through release-plz (`.github/workflows/release.yml`),
whose release pull request #841,
"chore: release v5.0.2",
has been open since 2026-08-13 and was last refreshed 2026-10-23.
The data reaches crates.io only when a maintainer merges it;
the crate has one crates.io owner (`Boshen`).

## Facts carried into the in-repo implementation

- The seven lines touch only `caniuse-lite` agent data,
  `node-releases` `envs.json`,
  and the Firefox ESR constant.
- `caniuse-lite` ships its agents as packed JavaScript,
  not JSON:
  `data/agents.js` starts `module.exports={A:{A:{K:0,D:0,...},B:"ms",C:["",...`,
  with keys decoded through `data/browsers.js` (`A:"ie",B:"edge",...`)
  and `data/browserVersions.js` (`"0":"117",...,A:"10",...`),
  as `dist/unpacker/agents.js` shows.
- `node-releases` ships strict JSON (`data/processed/envs.json`).
- The repository's pnpm install currently holds `browserslist@4.29.5`,
  `caniuse-lite@1.0.30001820`,
  and `node-releases@2.0.61` under `node_modules/.pnpm/`.
- The repository's 26 `Cargo.lock` files contain `flate2` and `tar` only transitively,
  and no HTTP client.

## Not done

- Regenerating oxc's data with `cargo codegen` from the repository's locked packages,
  which would have shown whether only the ESR constant then differs.
  Dropped when the user chose the in-repo implementation.
- No upstream issue was filed about the ESR constant;
  the in-repo implementation makes it moot for this repository.

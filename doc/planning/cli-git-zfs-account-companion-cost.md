# What the account companion costs one add

Measured 2026-11-24 on this host,
for step 10's deadline and for the ratio the design asks M11 to report
(`doc/planning/cli-git-zfs-start-probe-design.md`,
"The account companion",
"What it costs per add,
and what sizes that";
M11).

## What was measured, and why it is two terms

The design's ratio is "the same add with the companion disabled,
interleaved in
the same round".
This host cannot serve the ZFS route the companion sits in:
`package/git-policy/cli/src/native/worktree_clone_add_route/add_route_gate.rs`
serves only a checkout on the design's own dataset,
and plan step 19's migration is what puts this checkout there.
So the measurement times the two terms of that ratio,
in one round and on one build:

- the real `git worktree add --no-checkout` through this build's release wrapper
  (`cargo build --release --bins`) in a fixture repository with one commit;
- the companion's own run,
  exactly as the add's preflight makes it
  (`companion_parent::run_under(Switch::None, COMPANION_DEADLINE)`).

The identical-binary control is the second copy of each arm in the same round,
and the ratio reported stands only above the difference between two copies of
one command.

The measurement is
`package/git-policy/cli/src/native/account_companion/companion_cost_measurement_tests.rs`,
an ignored test:

```sh
CLI_GIT_ACCOUNT_COMPANION_MEASUREMENT_ROOT=<a scratch folder> \
  cargo test --release --lib account_companion -- --ignored --nocapture companion_cost_measurement
```

Five rounds,
three samples per arm per round,
arms interleaved
(add,
companion,
add,
companion,
preflight without the companion).

## The numbers

Per round,
medians in milliseconds:

- add:
  14.041,
  13.664,
  12.112,
  11.399,
  11.353;
- companion:
  1.729,
  1.791,
  1.665,
  1.643,
  1.622;
- ratio (add with the companion over the add):
  1.123104,
  1.131054,
  1.137493,
  1.144170,
  1.142859.

Over every sample of the round:

- add:
  8.914 to 91.366 ms.
  The 91.366 ms sample is the first one,
  a cold process and a cold page cache;
  every later sample is inside the round-to-round band.
- companion:
  1.342 to 2.759 ms.
- the preflight with the companion disabled:
  below the timer's resolution,
  0.000 ms at three decimals.

The companion therefore costs 0.12 to 0.14 of an add on this host,
and every companion sample is 0.0012 to 0.0028 seconds.

## The deadline this sizes

`companion_deadline.rs` ships
`COMPANION_DEADLINE = 100 ms`,
which is the slowest measured run (2.759 ms) plus
`COMPANION_DEADLINE_MARGIN = 97 ms`.

The margin is stated on its own because rule `LQM` asks for the measurement
beside the value and not hidden inside it.
97 ms is 68 times the measured spread of 1.42 ms,
and it is sized for this host's cold and loaded samples rather than for the
median:
the same round's first add ran 91.4 ms against an 11.4 ms median,
an excursion of about eight times,
and this host is known to load heavily
(`AGENTS.md`,
rule `NHR`).
A wedged mount call is still refused well inside a second,
and the bound covers the fork,
the setup,
the arms and the second child.

## What this measurement does not cover

- It is not a whole add through the ZFS route,
  so it does not include the
  route's own steps beside the companion.
  The ratio it reports is the companion's cost against a real add on this
  host,
  not against a managed add on a guest.
- It ran on this host only.
  The guest arm of M11 in this round reports the arm's own evidence (the skip
  on a guest whose profile denies the capabilities inside a namespace),
  not a
  second timing.
- The preflight without the companion is reported at 0.000 ms because it does
  no work at all;
  the arm exists so a ratio cannot be reported without a
  denominator measured in the same round.

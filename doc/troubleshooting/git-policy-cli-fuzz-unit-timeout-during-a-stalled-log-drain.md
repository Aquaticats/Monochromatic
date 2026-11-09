# libFuzzer (libfuzzer-sys 0.4.13) reports a unit timeout when a container's log drain stalls

## Symptom

`mise run //package/git-policy/cli.fuzz:smoke -- --seconds-per-target=900`
reported the `wrapper_controls` target as failed after 336,090 executions:

```text
wrapper_controls: FAILED, exit 70, 336090 executions; evidence .../campaign-tfBbHh/wrapper_controls
```

`exit.json` held `{"status":70,"signal":null,"executedUnits":336090}`,
and `artifacts/` held one `timeout-501b53c3274714c254041e7d72f5f393240f9a84`.
The end of `stderr.log` held libFuzzer's alarm and the saved unit:

```text
ALARM: working on the last Unit for 5 seconds
       and the timeout value is 5 (use -timeout=N to change)
0xf8,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x9f,
SUMMARY: libFuzzer: timeout
```

The timeout ended the target after 154 seconds of its 900,
so the three targets that ran before it were unaffected,
and this one spent 154 seconds on 336,090 units instead of its full time.

## Root cause

`package/git-policy/cli.fuzz/bin/fuzz-run.ts:180` passes libFuzzer a per-unit time bound:

```ts
`-timeout=${String(unitTimeoutSeconds)}`,
```

and `package/git-policy/cli.fuzz/bin/fuzz-run.ts:228` starts the container with
`podman start --attach`,
whose standard output and standard error are the evidence log files.
The fuzzer's own diagnostics therefore leave the container through the podman client,
which writes them into the evidence log on the host.

libFuzzer compares that bound against wall-clock time,
not against the unit's own work:
`FuzzerLoop.cpp:288` to `:294` of libfuzzer-sys 0.4.13
(`/home/user/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libfuzzer-sys-0.4.13/libfuzzer/FuzzerLoop.cpp`,
the same file the vendored copy builds):

```cpp
  size_t Seconds =
      duration_cast<seconds>(system_clock::now() - UnitStartTime).count();
  if (Seconds == 0)
    return;
  if (Options.Verbosity >= 2)
    Printf("AlarmCallback %zd\n", Seconds);
  if (Seconds >= (size_t)Options.UnitTimeoutSec) {
```

and `FuzzerFlags.def:55` of the same crate makes a timeout exit with status 70:

```text
FUZZER_FLAG_INT(timeout_exitcode, 70, "When libFuzzer reports a timeout "
```

So every second a unit waits for the host to drain its standard error is charged to that unit.

The stack libFuzzer printed,
symbolized with `addr2line` against the binary pulled from the run image,
shows where the alarm found the unit:

```text
#9  <std::sys::fd::unix::FileDesc>::write
#11 git_policy_cli::commit::commit_command::write_stderr
#12 git_policy_cli::invocation::wrapped_command::run_wrapped_command
#13 git_policy_cli_fuzz::controls::check_frontier
```

The unit was inside `write(2)` on its standard error,
which is the stream the host drains.

Two earlier readings are wrong.
The saved unit is not a slow input:
copied out of the evidence directory and run on the host with its standard error on a file,
the same 14 bytes finished in 0.151 seconds and wrote 1,797 bytes.
The container was not starved of CPU either:
`libpod-8c55569b....scope: Consumed 2min 51.934s CPU time over 3min 14.243s wall clock time`
is 88 percent of one CPU over its whole life.
Other sessions were building containers through `podman` at the time:
the journal of 19:15 to 19:19 holds several `podman run ... ubuntu:latest` scopes running `apt-get`
and `curl`,
each holding 1.5 to 3.5 CPU-seconds per wall second.

## Verification

Versions:
libfuzzer-sys 0.4.13 (`package/git-policy/cli.fuzz/Cargo.lock`),
cargo-fuzz 0.13.2,
podman 6.1.3,
rustc 1.101.0-nightly (19833af72 2026-10-20).

The failing case is the campaign itself:
one unit of `wrapper_controls` in its container,
its streams drained by `podman start --attach`,
with `-timeout=5`,
while other sessions built containers.

The passing case used the same corpus and the same container path,
with the fuzzer's two streams on host files and no other session building:

```bash
podman create --init --network=none --memory 2g --cpus 2 --pids-limit 512 \
  localhost/git-policy-cli-fuzz-run:<tag> /fuzz/bin/wrapper_controls /fuzz/corpus/wrapper_controls \
  -runs=340000 -max_len=4096 -rss_limit_mb=1536 -timeout=5 -print_final_stats=1 \
  -artifact_prefix=/fuzz/artifacts/wrapper_controls/ -dict=/fuzz/dictionary/controls.dict
podman start --attach <id> >stdout.log 2>stderr.log
```

It finished 340,000 runs in 176 seconds with exit status 0 and `stat::slowest_unit_time_sec: 0`.

The unit alone,
on the host,
with its standard error on a file:

```bash
./wrapper_controls -runs=1 -timeout=30 timeout-501b53c3274714c254041e7d72f5f393240f9a84
```

It finished in 0.151 seconds with exit status 0.

## Verified workarounds

Give the fuzzer a file instead of the container's drained stream.
The unit then runs in 0.151 seconds and the whole corpus behaves as above.
The width of the wait that this removes is the podman client's,
so the tradeoff is that a campaign run this way cannot read the log while it runs.

The fix adopted here keeps the drain and raises the bound instead.
`package/git-policy/cli.fuzz/bin/fuzz-run.ts:44` holds `unitTimeoutSeconds = 60`,
so a unit is reported only after 60 seconds,
which is 60 times the slowest unit any target of the campaign measured (`stat::slowest_unit_time_sec: 0`).
The tradeoff:
a target that really hangs is reported 55 seconds later than before,
and a drain that stalls for longer than 60 seconds would still fail a target.

## What does not work

- Reading the artifact as a slow input:
  the same bytes run in 0.151 seconds with a file as their standard error.
- Reading exit status 70 as a crash:
  libFuzzer's crash exit code is 77,
  and the artifact is named `timeout-`,
  not `crash-`.
- Trusting one clean rerun to prove the drain never stalls:
  the first stall needed a host busy with other sessions.

## The same target on 2026-10-15

A long campaign of 2026-10-15 also stopped in `wrapper_controls`:
the fourth container it started died after 152 seconds,
and the unit exited 1 after 57 minutes.
Its log is gone,
so whether that was this timeout is not established.
This record is what the 2026-11-08 run measured.

## Upstream filing artifact

Nothing to add.
The wall-clock bound and exit status 70 are libFuzzer's documented behavior
(`FuzzerFlags.def:55`),
and the wait this campaign hit is at our container boundary,
not in libFuzzer.

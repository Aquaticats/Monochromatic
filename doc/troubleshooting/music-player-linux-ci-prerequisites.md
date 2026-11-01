# Music player Linux CI needs its configured libclang path and an explicit headless compositor

## Scope and evidence

These are separate prerequisite incidents observed on application source
`e02be6c8b8fe9efb41a7f8f1a0b311c17796588a` on 2026-10-18.
The research harness changed disposable worker prerequisites and consumer activation only.
It did not change application assertions.
The allocation identities and finite resource bounds are retained in
[the coverage research](../planning/depot-complete-coverage-research.md)
and its machine-readable budget ledger.

## Bindgen cannot find the configured libclang directory

The actual desktop release producer in Depot CI run `9zvpr3t592`,
job `2mkshtfs6m`,
attempt `sjm3wrr50d`,
first stopped with bindgen `0.72.1` at `lib.rs:616`:

```text
Unable to find libclang: "couldn't find any valid shared libraries matching:
['libclang.so', 'libclang-*.so', 'libclang.so.*', 'libclang-*.so.*'],
set the `LIBCLANG_PATH` environment variable to a path where one of these files can be found (invalid: [])"
```

`package/music-player/desktop-app/mise.toml` defines `hostEnv()` for the Linux host branch.
It supplies `LIBCLANG_PATH: '/usr/lib64'` unconditionally.
The native macOS and Windows branch uses the process environment instead.
The installed bindgen source calls `clang_sys::load().expect("Unable to find libclang")`;
clang-sys `1.8.1` searches its supplied library-path variable in `build/common.rs:266`.

The prepared Ubuntu worker had its real library at `/usr/lib/llvm-18/lib/libclang.so`.
A disposable worker symlink from `/usr/lib64/libclang.so` to that existing library
allowed the unchanged desktop release task to complete.
The resulting executable was `53,526,792` bytes with SHA-256
`aa3bf32840df2d74856e4f519272580fb79a488caf99c0824065cbda2d696274`.
This qualifies that worker prerequisite bridge;
it does not establish a source fix or a universal library location.

## The default compositor requires a parent window system

The same producer run invoked the actual desktop E2E owner after building compositor `0.6.0`.
The owner stopped before its first check:

```text
CheckFailureError: the compositor socket never appeared
```

The compositor log selected `NestedWinit` and reported a parent Wayland display,
then failed to initialize winit.
`package/nested-wayland-session/cli/src/backend_select.rs` chooses the nested winit backend
unless `--headless` is explicit.
That default requires a usable parent window system.
The disposable CI runtime had no live parent display.

`package/music-player/desktop-e2e/src/config.ts` supports `E2E_COMPOSITOR` as an executable override.
A research forwarder supplied the actual immutable compositor executable with `--headless` added,
and forwarded its real `--version` output.
The compositor remained `9,220,128` bytes with SHA-256
`1508c92cb46135fcbd420aad50e046169d77f5aa89f07faf67925a14eb99b955`.
The consumer used software rendering,
a private PipeWire sink,
and a disposable systemd runtime bounded to 2 CPU,
2 GiB,
512 PIDs,
and no network.

## Headless activation reaches the UI but does not make the owner green

Run `3125s62dwd`,
job `nzkhw0wsbp`,
attempt `htd1mcqk6t`,
used harness `8070a3c83ec41a63ae8614ffe93dfcdbcd3ae435`.
It restored those exact executables without rebuilding them.
The unmodified desktop E2E owner emitted 69 `PASS` outcomes,
one authored Wayland modifier-repeat skip,
and 20 PNG captures.
The checks exercised real keyboard input,
clipboard access,
menus,
search,
playback selection,
badges,
and screenshot comparisons.

The owner then threw `CheckFailureError` while waiting for the thirty-track folder.
The stack names `src/ui-state-log.ts:321` and `src/wide-interaction.ts:362`.
The final observed UI state still named folder `F`.
This source wait failure is separate from the libclang,
parent-display,
and forwarder-format incidents.
Its cause has not been established,
and sessions after the wide session were not executed by that invocation.

Cleanup restored the private sink names and observed zero active test units,
application processes,
and compositor processes.
The runtime container was absent and its overlay writes discarded.
The temporary single-artifact download credential was removed.

Artifact `01a14f43-320d-7a25-8de3-1d00d38d4552` is `1,274,818` bytes with SHA-256
`027bfe42ff2cbcba19dd705b5675867e0355d91a307f009a029b7d50adf586a4`.
Its 96 files expand to `26,811,016` bytes;
all recorded process-log hashes were verified after download.
The consumer separately recorded 635 materialized input files totaling `8,201,667` bytes.
Those working-byte receipts do not retrospectively prove the original producer's asset bytes.

## Independent remaining sessions and native-dialog assertion

Run `0jpk5bwpvs` executes the six remaining unchanged session bodies with an explicit research scheduler.
Structured source summaries contain 41 passing outcomes and one failed dialog assertion.
The recorder prints live outcomes and the final report separately; counting both doubles the result.
The failed `browse/ctrl-shift-o` check observes zero changed pixels and no dialog covering the player.
The focus-return check passes. No cause or repair is established for the absent covering dialog.
Narrow layout, auto-play, quit, state, and visual sessions finish with their source assertions passing.
The same cleanup checks restore sinks and verify zero active units, app/compositor processes,
and retained containers. The artifact contains 90 PNG files.
An additional workflow step names an undefined research `desktop:cleanup` task,
which is a separate harness failure from both the source assertion and the already passed in-task cleanup.

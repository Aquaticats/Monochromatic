# The gate arm's header lengths, measured in two guests

**Non-final,
by an agent,
unapproved.**
It records one round's runs of
`compatibility_check::live_gate_tests::the_running_modules_declaration_answers_what_the_arm_expects`
in an `mvm` guest of the step 23 CachyOS template and in a guest of `mvm`'s Ubuntu 24.04
template.
The owner declares the work complete,
and nothing here is approved.

The issue it answers is 6502.
The failure was found by the step 23 qualification run
(`doc/planning/cli-git-zfs-step-23-host-kernel-qualification-run.md`,
"Step 2's arms:
the live suites of the binding").

## What the checker promises

`src/compatibility_check/btf_header.rs` states the two header lengths the reader
accepts,
in its file doc:

> Two header lengths exist:
> the 24 bytes Linux 6.8 wrote,
> whose string section must end the file,
> and the 32 bytes a current tool writes,
> whose layout section may sit between the types and the strings.
> Both are read,
> and nothing infers a kernel release from either.

The same file names them `OLDEST_HEADER_BYTES` (24) and `LAYOUT_HEADER_BYTES` (32),
and `read_sections` accepts the first,
refuses a length between the two,
and reads the second or a longer one.
So the length is a property of the module's own type-information file,
not of the distribution that built the module,
and a check that answers `compatible` for either shape is right by its own statement.

## What the arm asserted

Before this change the `compatible` branch asserted
`compatible.module_header_bytes == 24`,
with the message "a 6.8 guest's tools write the oldest header".
That 24 was the length the Ubuntu 24.04 guest's `pahole` wrote,
not a property of the check,
so the arm could not call a module qualified on a guest whose tools write 32,
even where the checker's own verdict was `compatible`.

The step 23 run met exactly that,
on the packaged `linux-cachyos-zfs` module of kernel `7.3.5-1-cachyos`:

```text
MEASUREMENT gate expected=compatible release=2.4.4-1 module_btf=570234 verdict=compatible base=6579400 module=570234
assertion `left == right` failed: a 6.8 guest's tools write the oldest header
  left: 32
 right: 24
```

## What changed

The arm now reads the header length out of the module file the checker read,
through the crate's own bounded read,
and parses the field itself instead of taking the checker's report of it,
so two independent readings of one file are compared.
It prints `MEASUREMENT gate-header declared=<n> reported=<n> oldest=24 layout=32`
and asserts:

- the length the module's own file states is `OLDEST_HEADER_BYTES` or
  `LAYOUT_HEADER_BYTES`;
- the length the checker reports is that same value.

No length is written into the test.
Commits `0bb2fc7fef` (the arm) and `7662012a15` (the arm's comment and the recipe's
"step 6 gate arms" section) carry it.

## The two guests

Both guests were made,
used and destroyed through `mvm --backend libvirt`,
one at a time,
and both ran the crate built inside the guest with
`cargo build --release --tests --locked`,
then the one arm through

```text
cargo nextest run --release --locked --run-ignored all --no-fail-fast --no-capture \
  --filterset "test(live_gate_tests)"
```

with `MONOCHROMATIC_ZFS_FIXTURE_GUEST` naming the guest and
`MONOCHROMATIC_ZFS_FIXTURE_GATE_EXPECT=compatible`.

The CachyOS guest was made from the step 23 template
(the `cachyos-7.3.5-1-zfs-2.4.4` image the qualification record describes),
renamed to its `mvm` name,
and given the 6 GiB `/swapfile` the template needs for a build:

- `uname --kernel-release` `7.3.5-1-cachyos`;
- `/sys/module/zfs/version` `2.4.4-1`;
- `/sys/kernel/btf/zfs` 570234 bytes,
  header field 32;
- `/sys/kernel/btf/vmlinux` 6579400 bytes.

The Ubuntu guest was `mvm`'s Ubuntu 24.04 image,
prepared by the package's own `prepare` step
(`e2e/run-fixture.ts`,
which installs the packages,
builds OpenZFS 2.4.4 against the guest's kernel headers with BTF,
and installs Rust and `cargo-nextest`):

- `uname --kernel-release` `6.8.0-147-generic`;
- `/sys/module/zfs/version` `2.4.4-1`;
- `/sys/kernel/btf/zfs` 579778 bytes,
  header field 24;
- `/sys/kernel/btf/vmlinux` 6103569 bytes;
- `pahole` v1.25.

## The runs

Every run names the `test result` line it printed.

- Red,
  before the change,
  in the CachyOS guest:
  `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 258 filtered out`,
  with `left: 32` and `right: 24`.
- Green,
  after the change,
  in the CachyOS guest:
  `MEASUREMENT gate-header declared=32 reported=32 oldest=24 layout=32` and
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 258 filtered out`.
- Red again with the old pin put back,
  in the same CachyOS guest,
  the whole pre-change file restored from the commit before the fix,
  rebuilt and run:
  `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 258 filtered out`,
  with the same `left: 32` and `right: 24`.
  That is the control showing the arm can fail.
- Green again after restoring the fixed file and rebuilding:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 258 filtered out`.
- Green in the Ubuntu guest,
  built from the fixed tree:
  `MEASUREMENT gate-header declared=24 reported=24 oldest=24 layout=32` and
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 258 filtered out`.

The two guests were destroyed through `mvm` after their runs,
and `mvm list` named neither of them afterwards.

## What did not run here

The arm alone ran in each guest.
The recipe's full root suite and account suite,
the step 6 change arms that rebuild and reboot the guest,
and the container gate did not run in this round,
and the two builds above were driven by hand inside the guests rather than through
the recipe's `run` step.
The crate's host checks ran once on this machine
(`cargo fmt`,
`cargo check --locked --offline --all-targets`,
`cargo clippy --locked --offline --all-targets -- -D warnings`,
and `cargo test --locked --offline`),
and no host command opened `/dev/zfs`.

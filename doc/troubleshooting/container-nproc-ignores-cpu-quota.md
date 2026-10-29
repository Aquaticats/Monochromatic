# Podman 5 on cgroup v2: `nproc` in a `--cpus` container prints the host's CPUs, while Rust programs read the quota

## Symptom

Inside a container that podman starts with `--cpus=2`,
`nproc` prints the CPU count of the host and not the two CPUs the container may use:

```text
$ podman run --rm --cpus=2 localhost/monochromatic/ide sh -c 'nproc; cat /sys/fs/cgroup/cpu.max'
32
200000 100000
```

`nproc` is the first thing a reader tries,
and it suggests that every tool in the container sizes itself to 32 CPUs on a quota of 2.
The IDE package (`package/desktop-app/ide`) suspected that of its test runner:
`cargo nextest run` would start one test thread per host CPU,
32 threads on a quota of 2 CPUs,
and the throttled threads would slow timing-sensitive tests.

That suspicion is wrong for the installed test runner.
A probe of 48 tests that each sleep four seconds,
run under `cargo nextest run` in the same container,
recorded the start and end time of every test and computed the most tests alive at once:

```text
old bound (--cpus=2, no NEXTEST_TEST_THREADS): nproc 32, cpu.max 200000 100000, most test processes at once: 2
```

The runner already used two test threads.
What is true is narrower:
tools that count CPUs with `sched_getaffinity` or `sysconf` (`nproc`,
`make -j$(nproc)`,
and C programs that call
`get_nprocs`) see the host,
and tools that read the control group quota see the limit.
Which kind a tool is decides whether it oversubscribes.

## Root cause

A `--cpus` limit is a CFS bandwidth quota in the container's control group (`cpu.max`),
not a restriction of the CPU set.
The container's threads may run on every host CPU,
so the affinity mask that `nproc` reads still lists all of them.

Rust's `std::thread::available_parallelism` takes the smaller of the affinity count and the control group quota.
The deciding code of the standard library,
at `rust-lang/rust` commit `99055eee3a49e1bca06855cd3f066384728a370a`:

```text
// library/std/src/sys/thread/unix.rs:172
let mut quota = usize::MAX;
quota = cgroups::quota().max(1);
...
let count = count.min(quota);
```

`cgroups::quota` reads the `cpu.max` file of the process's own control group
(`quota_v2` at `library/std/src/sys/thread/unix.rs:908`).
`cargo-nextest` sizes its default thread count from that function.
At the tag `cargo-nextest-0.9.143`,
the version in the build image,
`nextest-runner/src/config/core/imp.rs:191` reads:

```text
// nextest-runner/src/config/core/imp.rs:191
pub fn get_num_cpus() -> usize {
    static NUM_CPUS: LazyLock<usize> =
        LazyLock::new(|| match std::thread::available_parallelism() {
```

The nextest documentation (`site/src/docs/running.md`) names the same function as the meaning of the default
`num-cpus`.
The standard library of the image is Rust 1.98.0,
and the probe above confirms the behavior at runtime.

## Verification

The probe is a crate of 48 tests that each record their start and end time in milliseconds in a file and sleep
four seconds.
The most tests alive at once is the number of test threads the runner really used.
It ran under `podman run` with the limit arguments that `bin/container-bounds.ts` produces:

- With the old fixed bound (`--cpus=2`) and no `NEXTEST_TEST_THREADS`:
  `nproc` 32,
  `cpu.max` `200000 100000`,
  two tests at once.
- With the computed default of the build host (75 percent of 32 CPUs,
  so `--cpus=24`) and `NEXTEST_TEST_THREADS=24`:
  `nproc` 32,
  `cpu.max` `2400000 100000`,
  24 tests at once.
- With the same default and without `NEXTEST_TEST_THREADS` (the control):
  `nproc` 32,
  `cpu.max` `2400000 100000`,
  24 tests at once.
  The variable is redundant for this runner.
- With `IDE_CONTAINER_CPUS=2 IDE_CONTAINER_MEMORY=2g`:
  `nproc` 32,
  `cpu.max` `200000 100000`,
  two tests at once.
- With `IDE_CONTAINER_CPUS=2 IDE_CONTAINER_TEST_THREADS=8` (an override above the quota):
  `nproc` 32,
  `cpu.max` `200000 100000`,
  eight tests at once.

## Verified fix

`bin/container-bounds.ts` sets `NEXTEST_TEST_THREADS` and `CARGO_BUILD_JOBS` to the whole CPUs of the container
limit for every container the IDE tasks and scripts start.
This makes the thread count explicit,
so it no longer depends on the runner version reading the control group,
and it lets a caller override it with `IDE_CONTAINER_TEST_THREADS`.
It did not change the thread count on the installed runner,
which already matched the quota.

The slow IDE test suite had another cause,
which the thread count did not touch.
The test `every_row_of_real_source_fits_its_width` ran on one thread for 720 to 750 seconds.
Its helper `unexcused_breaks` called `space_advance(scale)` for every source line,
and `space_advance` in `tests/wrap.rs` builds a new `TextShaper`,
about 18,000 shapers for each of the six scale and width pairs.
The advance depends only on the scale,
so the pair now measures it once,
and the test is also split into six tests so that the pairs run side by side.
Together they cut the six tests from about 700 seconds on one thread to 8.7 seconds each at once with 24 CPUs;
the package README and `doc/handover/slint-ide-0x.md` record the numbers.

## Other containerised packages

The following packages pass `--cpus=` to podman or docker.
This note does not change them.
Whether one of them runs a tool that counts CPUs with the affinity mask (and so oversubscribes the quota)
needs the same probe per tool:

- `package/forbidden-strings/cli`
- `package/cli/nested-wayland-session`
- `package/config/pnpr`
- `package/desktop-app/terminal`
- `package/git-policy/cli`
- `package/git-policy/cli.fuzz`
- `package/linter/monochromatic-lint`
- `package/linter/monochromatic-lint.fuzz`
- `package/module/conf-fork`
- `package/module/deepmerge-ts.fuzz`
- `package/module/test`
- `package/music-player/desktop-app`
- `package/forbidden-strings/core`

## What does not work

- Reading `nproc` as the thread count of a Rust test runner.
  It prints 32 while nextest used 2.
- Lowering `NEXTEST_TEST_THREADS` below the quota to relieve throttling.
  The probe gave no evidence of throttling at the quota,
  and the IDE suite's long pole was one test whose cost was a shaper built per line.
- Raising the container bound alone.
  With the raised bound and the old test,
  `test` still took about 800 seconds,
  because the one test ran on one thread.

## Upstream filing decision

Nothing is filed.
`nproc` reports the affinity mask by design (coreutils documents it as the number of processing units available to
the current process),
and the runner's behavior is the correct one,
so there is no upstream defect to report.

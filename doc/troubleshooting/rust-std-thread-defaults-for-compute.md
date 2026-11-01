# Rust std 1.100 nightly on Linux: `available_parallelism` re-reads cgroups per call; spawned threads get 2 MiB

Two `std::thread` defaults surfaced
when `forbidden-regex` moved `RegexSet::new` rule construction onto worker threads
(`package/forbidden-strings/core/src/parallel.rs`).
Both are documented behavior, not bugs,
but each one silently regresses a library that adopts threads naively.

## Symptom

### Per-call cost of `available_parallelism`

After threading landed,
a single-pattern `RegexSet::compile_lenient` got slower even though one pattern never spawns a thread:

- before threading: 45.9, 47.4, and 46.3 microseconds per call
- after threading, calling `thread::available_parallelism()` on every call:
  111.9, 110.0, and 110.0 microseconds per call
- after caching the core count in a `OnceLock`: 45.6 microseconds per call

`forbidden-strings` validates each user rule with a one-element `RegexSet::new` call
(`package/forbidden-strings/cli/src/rule/frx.rs:111`),
so the uncached lookup would have added its cost once per rule.

### Stack overflow on deeply nested patterns in spawned threads

A pattern of nested non-capturing groups aborts the whole process
when it is built on a thread spawned with the default stack:

```text
thread '<unknown>' (2110670) has overflowed its stack
fatal runtime error: stack overflow, aborting
```

Under `cargo nextest` the same abort is reported as
`SIGABRT [   0.131s] (1/1) forbidden-regex parallel::tests::workers_build_deeply_nested_groups`
with `(test aborted with signal 6: SIGABRT)`.
It is not a panic, so no `catch_unwind` boundary can contain it.

## Root cause

### `available_parallelism` is intentionally uncached

The public docs state the cost directly,
`library/std/src/thread/functions.rs:633` to `:635` in the `rust-src` component of `nightly-2026-09-16`:

```rust
/// Resource limits can be changed during the runtime of a program, therefore the value is
/// not cached and instead recomputed every time this function is called. It should not be
/// called from hot code.
```

On Linux the implementation queries cgroup quotas before the affinity mask on every call,
`library/std/src/sys/thread/unix.rs:174` to `:179`:

```rust
            #[cfg(any(target_os = "android", target_os = "linux"))]
            {
                quota = cgroups::quota().max(1);
                let mut set: libc::cpu_set_t = unsafe { mem::zeroed() };
                unsafe {
                    if libc::sched_getaffinity(0, size_of::<libc::cpu_set_t>(), &mut set) == 0 {
```

`cgroups::quota` opens and parses `/proc/self/cgroup` each time,
`library/std/src/sys/thread/unix.rs:869` to `:872`:

```rust
        let _: Option<()> = try {
            let mut buf = Vec::with_capacity(128);
            // find our place in the cgroup hierarchy
            File::open("/proc/self/cgroup").ok()?.read_to_end(&mut buf).ok()?;
```

then, for cgroup v2, walks from the process cgroup up to `/sys/fs/cgroup`,
opening `cpu.max` at every level,
`library/std/src/sys/thread/unix.rs:929` to `:935`:

```rust
        let _: Option<()> = try {
            while path.starts_with(cgroup_mount) {
                path.push("cpu.max");

                read_buf.clear();

                if File::open(&path).and_then(|mut f| f.read_to_string(&mut read_buf)).is_ok() {
```

Each call is several file opens and reads plus `sched_getaffinity`,
which is the 64-microsecond gap measured in "Per-call cost of `available_parallelism`".
Caching was the only change between the 110 and 45.6 microsecond runs,
which ties the gap to this call.

### Spawned threads default to 2 MiB; the main thread does not

The module docs,
`library/std/src/thread/mod.rs:128` to `:138`:

```rust
//! The default stack size is platform-dependent and subject to change.
//! Currently, it is 2 MiB on all Tier-1 platforms.
//!
//! There are two ways to manually specify the stack size for spawned threads:
//!
//! * Build the thread with [`Builder`] and pass the desired stack size to [`Builder::stack_size`].
//! * Set the `RUST_MIN_STACK` environment variable to an integer representing the desired stack
//!   size (in bytes). Note that setting [`Builder::stack_size`] will override this. Be aware that
//!   changes to `RUST_MIN_STACK` may be ignored after program start.
//!
//! Note that the stack size of the main thread is *not* determined by Rust.
```

The constant, `library/std/src/sys/thread/unix.rs:33`:

```rust
pub const DEFAULT_MIN_STACK_SIZE: usize = 2 * 1024 * 1024;
```

The fallback when neither `Builder::stack_size` nor `RUST_MIN_STACK` is set,
`library/std/src/thread/lifecycle.rs:36` to `:38`:

```rust
        let amt = env::var_os("RUST_MIN_STACK")
            .and_then(|s| s.to_str().and_then(|s| s.parse().ok()))
            .unwrap_or(imp::DEFAULT_MIN_STACK_SIZE);
```

The main thread instead gets its stack from the `ulimit -s` resource limit,
which is `16384` KiB (16 MiB) in this repository's development shell.
`forbidden-regex` parsing has no nesting-depth cap and recurses once per group level,
so moving the same work from the main thread to a default-stack thread
cuts the survivable nesting depth by the ratio of the two stacks.

## Verification

Versions under test:

- `rustc 1.100.0-nightly (215a8af4b 2026-09-15)`, `rust-src` component of `nightly-2026-09-16-x86_64-unknown-linux-gnu`
- `cargo 1.100.0-nightly (7941be6fb 2026-09-11)`
- Linux `7.2.0-ogc6.1.fc44.x86_64`, 16 logical CPUs, cgroup v2

Harness crate (throwaway, outside the repository),
depending on the engine by path so the before and after commits can be swapped:

```toml
# frx-timing/Cargo.toml
[package]
name = "frx-timing"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
forbidden-regex = { path = "/var/home/user/Monochromatic/package/forbidden-strings/core" }

[profile.release]
opt-level = 3
lto = false
overflow-checks = true
panic = "unwind"

[workspace]
```

Stack-depth probe:

```rust
// frx-timing/src/bin/deep.rs
use forbidden_regex::RegexSet;

fn main() {
    let depth: usize = std::env::args().nth(1).expect("depth").parse().expect("depth number");
    let stack_mib: usize = std::env::args().nth(2).expect("stack MiB").parse().expect("stack number");
    let pattern = format!("{}abcdef{}", "(?:".repeat(depth), ")".repeat(depth));
    let handle = std::thread::Builder::new()
        .stack_size(stack_mib * 1024 * 1024)
        .spawn(move || {
            let start = std::time::Instant::now();
            let outcome = RegexSet::new(std::slice::from_ref(&pattern)).map(|set| set.len());
            (outcome.map_err(|e| e.to_string()), start.elapsed().as_secs_f64())
        })
        .expect("spawn");
    let (outcome, seconds) = handle.join().expect("join");
    eprintln!("depth {depth} stack {stack_mib}MiB: {outcome:?} in {seconds:.3}s");
}
```

Small-set cost probe:

```rust
// frx-timing/src/bin/small.rs
use forbidden_regex::RegexSet;
use std::time::Instant;

fn main() {
    let count: usize = std::env::args().nth(1).expect("pattern count").parse().expect("number");
    let rounds: usize = std::env::args().nth(2).expect("rounds").parse().expect("number");
    let owned: Vec<String> = (0..count).map(|n| format!("tok{n}_[A-Za-z0-9]{{8}}")).collect();
    let start = Instant::now();
    let mut kept_total = 0;
    for _ in 0..rounds {
        let (set, kept) = RegexSet::compile_lenient(&owned);
        kept_total += kept.len() + set.len();
    }
    let per_call = start.elapsed().as_secs_f64() / rounds as f64;
    eprintln!("patterns {count}: {:.1} us per compile_lenient ({kept_total})", per_call * 1e6);
}
```

```sh
cargo build --release
./target/release/deep 3000 2
./target/release/small 1 20000
```

### Stack configurations that build cleanly

- depth 1000 on a 2 MiB thread: `Ok(1) in 0.001s`
- depth 10000 on an 8 MiB thread: `Ok(1) in 0.003s`
- depth 10000 on a 64 MiB thread: `Ok(1) in 0.003s`

### Stack configurations that abort

- depth 3000 on a 2 MiB thread: `has overflowed its stack`, exit code 134
- depth 10000 on a 2 MiB thread: `has overflowed its stack`, exit code 134

### Small-set cost, before threading vs after caching the core count

Measured alternately in one session to share the noise band:

- 1 pattern: 46.3 before, 45.6 after
- 2 patterns: 99.6 before, 140.3 after (thread startup for two workers)
- 4 patterns: 178.3 before, 192.5 after
- 8 patterns: 343.2 and 340.2 before, 352.2 and 336.2 after (same band)

## Verified workarounds

### Cache the core count once per process

```rust
// package/forbidden-strings/core/src/parallel.rs
static CORE_COUNT: OnceLock<usize> = OnceLock::new();

fn core_count() -> usize {
    return *CORE_COUNT.get_or_init(|| return thread::available_parallelism().map_or(1, NonZeroUsize::get))
}
```

Tradeoff:
a cgroup quota or affinity change after the first compile is not observed,
so a process whose CPU budget shrinks later can oversubscribe.
That is the exact staleness the std docs chose not to accept by default.

### Give compute workers an explicit stack size

```rust
// package/forbidden-strings/core/src/parallel.rs
let spawned = thread::Builder::new()
    .stack_size(WORKER_STACK_BYTES)
    .spawn_scoped(scope, || return drain(patterns, stop_on_error, next, first_error));
```

`WORKER_STACK_BYTES` is 64 MiB.
Tradeoffs:
each worker reserves 64 MiB of address space
(committed only when touched, but a strict `vm.overcommit_memory` setting can refuse the spawn,
which the helper handles by continuing with fewer workers or building on the calling thread);
nesting beyond what 64 MiB holds still aborts the process;
a caller whose own main thread is larger than 64 MiB could see a lower limit than before.
The guard test `parallel::tests::workers_build_deeply_nested_groups` aborts with `SIGABRT`
when `.stack_size(WORKER_STACK_BYTES)` is removed, and passes with it.

## What does not work

- `RUST_MIN_STACK`:
  it is read once per process from the environment (`library/std/src/thread/lifecycle.rs:36`),
  so a library cannot set it for its own threads,
  and it would also change every unrelated thread the host program spawns.
- Running one worker on the calling thread in addition to the spawned ones:
  rejected by reasoning, not measurement.
  The calling thread's stack is unknown to the library
  (a `cargo test` thread is 2 MiB, a `build.rs` main thread follows `ulimit -s`),
  so which rule lands on it would make the depth limit nondeterministic,
  and the deep-nesting guard test could pass or abort depending on scheduling.
- Skipping `available_parallelism` only for one-pattern calls:
  it would still add 64 microseconds to every multi-pattern call,
  where the caching fix removes it for all calls.

## Upstream filing artifact

### Upstream filing decision

Checked `.out-of-scope/`: no entry covers Rust std or its thread API.

1. **Is it really upstream's fault?**
   No.
   Both behaviors are documented and deliberate:
   `library/std/src/thread/functions.rs:633` to `:635` explains why the count is recomputed every call,
   and `library/std/src/thread/mod.rs:128` to `:138` documents the 2 MiB spawn default and both override paths.
   The regression was ours for adopting threads without reading them.
2. **Can upstream fix it?**
   Not applicable: there is no defect to fix.
3. **Are they supporting this use case?**
   Yes, through `Builder::stack_size` and caller-side caching, which is what the workarounds use.
4. **Would the repo welcome our contribution?**
   Not evaluated, because constraint 1 fails.
5. **Will they likely fix it?**
   Not applicable.
6. **Have we prototyped a minimal fix?**
   Not applicable.

Duplicate search:
`gh search issues --repo rust-lang/rust 'available_parallelism'` returned
rust-lang/rust#98168
("compile-time regression from switch from num_cpus to available_parallelism needs a regression test"),
the closest related thread about the call's cost.
Nothing here advances it: our finding is a consumer calling a documented-uncached function per operation.

Nothing to file, and no comment to post.

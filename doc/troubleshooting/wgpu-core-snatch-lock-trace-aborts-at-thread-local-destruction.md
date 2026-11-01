# wgpu-core 30.0.1 debug builds abort at process exit when a thread-local value drops the last wgpu handles ("cannot access a Thread Local Storage value during or after destruction")

Tool under test:
wgpu-core 30.0.1 (crates.io checksum `14c018fce9b6270aa203c2fdd56f3cce996713534bd757e4ea58c8560b121f14`,
git commit `40f4a34ebaf56f9a046231f54125ad046239d3f3`,
tag `v30.0.1`) through wgpu 30.0.1,
on the Vulkan backend (RADV on an AMD RX 7600,
Mesa,
Fedora 44),
as used by Slint 1.18.1 in `package/ide/app`.
The study that found it is the section "An abort at exit in debug builds,
found and fixed" of
`doc/planning/ide-gpu-snapshot-harness.md`.

## Symptom

A debug build (`debug_assertions` on,
which a Cargo dev profile also applies to dependencies) ends with this
after the last window closes,
and the process exits with status 134:

```text
thread 'main' (3686974) panicked at .../library/std/src/thread/local.rs:436:25:
cannot access a Thread Local Storage value during or after destruction: AccessError
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
fatal runtime error: thread local panicked on drop, aborting
```

The IDE showed it in two of two debug runs of the new launcher (one on the RX 7600,
one on llvmpipe) and in none of its
release runs.
A release build never shows it,
because the code involved is compiled out when `debug_assertions` is off.
The same text,
from a different embedder,
is in `gfx-rs/wgpu` issue 8259 (closed,
see the duplicate search).

## Root cause

wgpu-core keeps a debug-only thread-local that records which snatch lock the thread holds,
so that a second acquisition on the same thread panics with both backtraces.
It is compiled in only for debug builds with the `std` feature,
and it is a thread-local with a destructor
(the stored `LockTrace` owns a `Backtrace`,
which frees memory when dropped):

```rust
// wgpu-core-30.0.1/src/snatch.rs:106-116
use trace::LockTrace;
#[cfg(all(debug_assertions, feature = "std"))]
mod trace {
    use core::{cell::Cell, fmt, panic::Location};
    use std::{backtrace::Backtrace, thread};

    pub(super) struct LockTrace {
        purpose: &'static str,
        caller: &'static Location<'static>,
        backtrace: Backtrace,
    }
```

```rust
// wgpu-core-30.0.1/src/snatch.rs:137-157
            if let Some(prev) = SNATCH_LOCK_TRACE.take() {
                ...
            } else {
                SNATCH_LOCK_TRACE.set(Some(new));
            }
        }

        pub(super) fn exit() {
            SNATCH_LOCK_TRACE.take();
        }
    }

    std::thread_local! {
        static SNATCH_LOCK_TRACE: Cell<Option<LockTrace>> = const { Cell::new(None) };
    }
```

`LocalKey::take` goes through `LocalKey::with`,
which panics with `AccessError` when the key's destructor has already
run on this thread.
Both `enter` and `exit` call it unconditionally,
and every read or write of the device's snatch lock calls them:

```rust
// wgpu-core-30.0.1/src/snatch.rs:188-191
    #[track_caller]
    pub fn read(&self) -> SnatchGuard<'_> {
        LockTrace::enter("read");
        SnatchGuard(self.lock.read())
```

Dropping a wgpu `Queue` takes that lock,
so a `Queue` that is freed after the thread-local was destroyed aborts:

```rust
// wgpu-core-30.0.1/src/device/queue.rs:275 and 296
impl Drop for Queue {
    ...
        let snatch_guard = self.device.snatchable_lock.read();
```

`Global::queue_drop` (`wgpu-core-30.0.1/src/device/global.rs:1994-1999`) removes the queue from the hub,
which is what runs that `Drop` when the last handle goes.
The backtrace of the panic,
from the harness below with `RUST_BACKTRACE=1`,
shows the chain:

```text
   3: <std::thread::local::LocalKey<Cell<Option<wgpu_core::snatch::trace::LockTrace>>>>::with::<...take, ...>
   5: <wgpu_core::snatch::trace::LockTrace>::enter   at wgpu-core-30.0.1/src/snatch.rs:137:51
   6: <wgpu_core::snatch::SnatchLock>::read           at wgpu-core-30.0.1/src/snatch.rs:190:9
   7: <wgpu_core::device::queue::Queue as core::ops::drop::Drop>::drop
                                                      at wgpu-core-30.0.1/src/device/queue.rs:296:56
  13: <wgpu_core::global::Global>::queue_drop         at wgpu-core-30.0.1/src/device/global.rs:1998:41
  14: <wgpu::backend::wgpu_core::CoreQueue as core::ops::drop::Drop>::drop
  27: std::sys::thread_local::native::eager::destroy::<RefCell<Option<tls_abort::Handles>>>::{closure#0}
```

Whether the key is already destroyed depends on destruction order.
Rust runs thread-local destructors in the reverse order of their registration,
and a key registers when it is first
accessed.
When the thread-local that owns the wgpu handles registers before `SNATCH_LOCK_TRACE` does,
`SNATCH_LOCK_TRACE` is destroyed first and the handles are dropped second.
That order is a property of the platform's thread-local implementation (glibc on this host),
not something the Rust language promises,
so the harness below pins it with a control that reverses it.

Slint is a case with the unlucky order.
Its platform,
and with it the winit backend,
the renderer and the device handles that the renderer holds,
lives in a thread-local that is touched when the platform is installed,
which is before the first frame takes the snatch lock:

```rust
// i-slint-core-1.18.1/context.rs:23-26 and 30
crate::thread_local! {
    pub(crate) static GLOBAL_CONTEXT : once_cell::unsync::OnceCell<SlintContext>
        = const { once_cell::unsync::OnceCell::new() }
}
...
    platform: Box<dyn Platform>,
```

`crate::thread_local!` is `std::thread_local!` when Slint's `std` feature is on (`i-slint-core-1.18.1/lib.rs:29-30`).
The native test harness of the IDE leaks its device for this reason,
and the launcher does the same (see the workarounds).
I did not capture a backtrace from the IDE itself for this document;
the IDE evidence is the recorded runs in `doc/planning/ide-gpu-snapshot-harness.md`,
and the chain from Slint's thread-local to the panic is read from the sources above plus the minimal program below.
The author of the upstream fix reports the same trigger in Slint (see the upstream filing decision).

Only the snatch lock's trace is compiled in by default.
The other two lock instrumentations keep a thread-local too
(`wgpu-core-30.0.1/src/lock/ranked.rs:87-89` behind `cfg(wgpu_validate_locks)`,
`wgpu-core-30.0.1/src/lock/observing.rs:308-310` behind the `observe_locks` feature),
and the upstream fix touches them as well;
this build enables neither,
and I did not test them.

## Verification

Versions:
wgpu and wgpu-core 30.0.1 from the registry,
`pollster` 0.4,
rustc 1.101.0-nightly (2026-10-17),
GPU RADV NAVI33.
The program owns the wgpu handles in a thread-local of the main thread,
as Slint does,
and registers that thread-local before wgpu touches its own.
No window system is needed.

```toml
# Cargo.toml
[package]
name = "tls-abort"
version = "0.1.0"
edition = "2024"

[workspace]

[dependencies]
wgpu = { version = "=30.0.1", default-features = false, features = ["vulkan", "wgsl", "std"] }
pollster = "0.4"
```

```rust
// src/main.rs
use std::cell::RefCell;

struct Handles {
    _instance: wgpu::Instance,
    _adapter: wgpu::Adapter,
    _device: wgpu::Device,
    _queue: wgpu::Queue,
}

thread_local! {
    // Registered before wgpu touches its own thread-local snatch trace, so the Rust runtime destroys
    // the trace first (reverse order of registration) and this value last.
    static HOLDER: RefCell<Option<Handles>> = const { RefCell::new(None) };
}

fn main() {
    let leak = std::env::args().any(|argument| argument == "--leak");
    // `--late` skips the early registration, which is the control: the holder then registers after the trace
    // and is destroyed first.
    if !std::env::args().any(|argument| argument == "--late") {
        HOLDER.with(|_| ());
    }
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
        .expect("an adapter");
    println!("adapter {:?}", adapter.get_info().name);
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
        .expect("a device");
    // One submit takes the snatch lock on the main thread, as any frame does.
    queue.submit([]);
    if leak {
        // The workaround: one extra handle of each object that is never dropped.
        Box::leak(Box::new((instance.clone(), adapter.clone(), device.clone(), queue.clone())));
    }
    HOLDER.with(|holder| {
        *holder.borrow_mut() = Some(Handles {
            _instance: instance,
            _adapter: adapter,
            _device: device,
            _queue: queue,
        });
    });
    println!("main returns");
}
```

```sh
# Run from the crate directory. Cargo needs the registry cache or network for the dependencies.
cargo build
./target/debug/tls-abort ; echo "exit=$?"
```

### Runs that fail

- `cargo build` then `./target/debug/tls-abort`:
  prints `adapter "AMD Radeon RX 7600 (RADV NAVI33)"` and
  `main returns`,
  then the panic text in the symptom,
  and exits with status 134 (`Aborted (core dumped)`).
  `RUST_BACKTRACE=1` gives the chain above.

### Runs that work

- `./target/debug/tls-abort --leak`:
  exit 0.
  This is the workaround in the IDE.
- `./target/debug/tls-abort --late`:
  exit 0.
  This is the control:
  with the holder registered after wgpu's thread-local,
  the destruction order is reversed and
  nothing aborts.
  It shows the abort needs that order,
  and that the program is not failing for another reason.
- `cargo build --release` then `./target/release/tls-abort`:
  exit 0.
  The trace code is compiled out.
- The same debug build with `[profile.dev.package.wgpu-core] debug-assertions = false` appended to `Cargo.toml`:
  exit 0.
- The same debug build with the upstream fix applied (see the prototype):
  exit 0.

## Verified workarounds

### Leak one extra handle of each wgpu object

`package/ide/app/src/launcher.rs:259-275` (the application) and
`package/ide/app/src/native/gpu_snapshot.rs:253-264` (the test harness) keep one more clone of the instance,
adapter,
device and queue in a leaked `Box`,
so the last handle is never dropped at thread exit:

```rust
// package/ide/app/src/launcher.rs:270-275
    Box::leak(Box::new((
        gpu.instance.clone(),
        gpu.adapter.clone(),
        gpu.device.clone(),
        gpu.queue.clone(),
    )));
```

Tradeoffs:

- The device,
  queue and everything they own are never destroyed by wgpu;
  the operating system frees the memory at exit.
  Nothing in the process shows a Vulkan object leak report,
  because no Vulkan validation layer is installed on this
  host (`/usr/share/vulkan/explicit_layer.d/` is empty);
  with `VK_LAYER_KHRONOS_validation` the leaked objects would
  be expected to appear in its exit report,
  and I did not check that.
- Every code path that creates a second device in the same process needs its own leak.
- The leak is needed in release builds as well only if a future wgpu release adds the same pattern to release code;
  it is harmless there.

### Turn debug assertions off for wgpu-core only

```toml
# Cargo.toml of the application workspace
[profile.dev.package.wgpu-core]
debug-assertions = false
```

Verified on the program above:
the debug build exits 0.
Tradeoffs:

- wgpu-core loses its own debug checks in dev builds,
  including the recursive snatch-lock detector this trace exists
  for.
- It changes the dev profile of the whole workspace,
  and it is a build setting that someone has to know about,
  while the leak sits in the code that causes the order.

### Use the upstream fix

The fix is on `gfx-rs/wgpu` trunk (merge commit `efcf445784dd31537f09bc61bad27431de713fce`,
2026-10-17) and not in a
release:
`v30.0.1` (2026-08-22) is the latest release,
and trunk is 631 commits past it.
Depending on a git revision of `wgpu-core` together with matching `wgpu`,
`wgpu-hal`,
`wgpu-types` and the naga
crates,
as the prototype below does with `[patch.crates-io]`,
would carry it today.
Tradeoff:
Slint 1.18.1 depends on the registry release of wgpu 30,
so the patch has to cover every wgpu crate at once;
I did not try that against the IDE,
only against the program above.

## What does not work

- Registering the owning thread-local late (the `--late` run) is not a fix.
  It only reverses the order for one program,
  and the order is decided by which key a thread touches first,
  which an embedder such as Slint does not control.
- Ignoring the abort because it happens at exit:
  the process exit status is 134,
  a core dump is written,
  and a test runner or a supervisor counts it as a crash.
- Release builds hide it,
  so a release-only check does not show that the order is safe.

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` has no entry for wgpu or Slint (entries read:
`bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
`typescript-project-references.md`;
`cargo-workspace.md` mentions Slint only as a library to keep off the host),
so filing would not be out of scope.

Duplicate search,
`gh search issues --repo gfx-rs/wgpu` and `gh search prs --repo gfx-rs/wgpu`,
open and closed:

- `"Thread Local Storage"`:
  issue 8259 "Help with understanding LockTrace::enter panic" (closed).
  The reporter embeds wgpu-native from C++ and gets this exact `AccessError`;
  the maintainer (jimblandy) answered that the shutdown order is probably the embedder's and asked for a Rust program
  that shows it before reopening.
  The program above is such a program.
- `"LockTrace"`:
  issue 9250 "`LockTrace::enter` causes terrible performance on debug (creates many backtraces)" (open),
  unrelated to destruction.
- `"snatch"` returned a page of issues (open and closed),
  none about thread-local destruction
  (deadlocks,
  recursion with `as_hal`,
  mapping races).
- `"thread local panicked on drop"` and `"SNATCH_LOCK_TRACE"` and `"during or after destruction"`:
  no issue other than 8259.
- `prs "LockTrace"`:
  pull request 10030 "Fix crash in lock validation due to TLS destructor call order",
  **merged 2026-10-17**.
  Its description names this cause and this embedder:
  "I personally observed this issue while moving slint's skia surface creation code to use wgpu" (Slint pull request
  12819,
  merged 2026-08-31).
  The merged change replaces `SNATCH_LOCK_TRACE.take()` in `enter` and `exit` with
  `SNATCH_LOCK_TRACE.try_with(...)` (and the same in `lock/ranked.rs` and `lock/observing.rs`),
  so lock tracing stops silently once the key is gone.
  The review thread records that a `ManuallyDrop` variant was tried and dropped for `try_with`
  ("it seems safer to just give up on lock tracing during thread exit").
  It adds a test,
  `read_during_tls_destruction`,
  whose shape is the program above.
- `gh search issues --repo slint-ui/slint "wgpu thread local destruction"`:
  no results.

Candidate-fix gate:
the installed `wgpu-core-30.0.1/src/snatch.rs:137-153` still has the unguarded `take()` calls,
the issue's symptom and stack are the same as the observed failure,
and the fix's own test has the same shape as the harness,
so the fix applies to this incident and is simply not
released.

The six constraints:

1. **Is it really upstream's fault?**
   Yes,
   as wgpu's maintainers decided it:
   they treated an abort in an instrumentation-only path as a bug and merged a
   fix.
   The destruction order is the embedder's to some degree (issue 8259's answer),
   but debug instrumentation that turns a legal drop into an abort is wgpu-core's to harden.
2. **Can upstream fix it?**
   Yes,
   and did:
   a `try_with` in `enter` and `exit`.
3. **Are they supporting this use case?**
   Yes:
   the merged change names Slint as the motivating embedder.
4. **Would the repo welcome our contribution?**
   Not applicable now (nothing to contribute).
   For the record:
   `CONTRIBUTING.md` section "LLMs (AI)" says "Using LLMs and AIs to generate code that is part of a
   contribution is allowed" and asks that "Issue and PR descriptions and large explanatory comment blocks" be written
   by the author;
   `.github/ISSUE_TEMPLATE/` has bug,
   feature and other templates;
   no ban was found.
   Any filing here would need a human-written description.
5. **Will they likely fix it?**
   Already fixed on trunk;
   the release carrying it is not out.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   Yes,
   by applying upstream's own `snatch.rs` hunk to a fresh clone and running the program against it.

### Prototype

Disposable clone:
`gh repo clone gfx-rs/wgpu <dir> -- --depth 1 --branch v30.0.1`,
`origin` is
`https://github.com/gfx-rs/wgpu.git`,
`HEAD` is `40f4a34ebaf56f9a046231f54125ad046239d3f3`
(matches `wgpu-core-30.0.1/.cargo_vcs_info.json` in the registry copy).
The patch is the `wgpu-core/src/snatch.rs` part of pull request 10030 (`git apply --include`);
the program's `Cargo.toml` got a `[patch.crates-io]` table pointing `wgpu`,
`wgpu-core`,
`wgpu-hal`,
`wgpu-types`,
`wgpu-naga-bridge`,
`naga` and `naga-types` at the clone.
The part that matters:

```diff
--- a/wgpu-core/src/snatch.rs
+++ b/wgpu-core/src/snatch.rs
-            if let Some(prev) = SNATCH_LOCK_TRACE.take() {
-                ...panic!("... attempted to acquire a snatch lock recursively ...")
-            } else {
-                SNATCH_LOCK_TRACE.set(Some(new));
-            }
+            // try_with() because lock may be unavailable due to TLS destruction call order
+            let _ = SNATCH_LOCK_TRACE.try_with(|trace| {
+                if let Some(prev) = trace.take() { ...panic!(...) } else { trace.set(Some(new)); }
+            });
...
         pub(super) fn exit() {
-            SNATCH_LOCK_TRACE.take();
+            let _ = SNATCH_LOCK_TRACE.try_with(|trace| trace.take());
         }
```

Before:
`./target/debug/tls-abort` exit 134 with the panic text.
After:
`adapter "AMD Radeon RX 7600 (RADV NAVI33)"`,
`main returns`,
exit 0.
The two other files in the pull request (`lock/ranked.rs`,
`lock/observing.rs`) were not applied and are not needed
for this program.

### Decision

Do not file anything.
The behavior is already fixed upstream by the pull request above,
so a new issue would be a duplicate,
and a comment on the pull request would not add anything the thread lacks:
it already names the Slint trigger,
the cause and the fix.
The one open question,
when a release carries the fix,
is the maintainers' to answer and is visible in the releases
list.
Draft:
none;
an explicit "nothing to add" note.

Follow-up for the repository:
when a wgpu release newer than 30.0.1 that contains commit
`efcf445784dd31537f09bc61bad27431de713fce` is the one Slint resolves,
remove the leak in `package/ide/app/src/launcher.rs` and `src/native/gpu_snapshot.rs` and run a debug build
to confirm the exit is clean.

## Evidence and limits

- Reproduced and fixed with the program above on wgpu 30.0.1,
  one GPU (RADV),
  glibc thread-local destruction order.
- Not reproduced inside the IDE for this document,
  and no IDE backtrace was taken.
- The rank and observing lock instrumentations were not tested.
- The study with the surrounding context is `doc/planning/ide-gpu-snapshot-harness.md`
  (section "An abort at exit in debug builds,
  found and fixed").

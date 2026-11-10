# `@ff-labs/pi-fff` 0.9.6 live grep causes a native SIGBUS in Pi

## Symptom

Pi exits with:

```text
Bus error (core dumped) pi
```

The terminal remains in Pi's keyboard protocol after the crash.
 The shell then
interprets terminal response bytes as commands,
 producing strings such as:

```text
bash: command not found: pi2
bash: command not found: 1:3u5
```

The coredump reports signal `7 (BUS)`.
 Its native stack names `libfff_c.so`,
`ffi-rs.linux-x64-gnu.node`,
 and `fff_live_grep`.

## Root cause

The auto-mode ask notification is not the crashing process.
 The coredump's main
Pi thread is inside FFF's native live-grep path,
 not `nano-spawn`,
 `notify-send`,
or auto-mode.
 Installed package metadata identifies `@ff-labs/pi-fff` and
`@ff-labs/fff-node` as version `0.9.6`.

The v0.9.6 source trace is:

1.  `package/pi-fff/src/index.ts:633-645` calls `f.grep(...)` for Pi's
    `ffgrep` tool:

    ```ts
    const grepResult = f.grep(query, {
      mode,
      smartCase,
      maxMatchesPerFile: Math.min(effectiveLimit, 50),
    });
    ```

2.  `package/fff-node/src/ffi.ts:1352-1397` maps that call to native
    `fff_live_grep` through `ffi-rs`:

    ```ts
    const rawPtr = load({
      library: LIBRARY_KEY,
      funcName: "fff_live_grep",
    });
    ```

3.  `crates/fff-core/src/file_picker.rs:1199-1220` runs grep inside FFF's native
    search thread pool:

    ```rust
    SEARCH_THREAD_POOL.install(|| {
        grep_search(
            self.get_files(),
            query,
            options,
    ```

A direct `notify-send` probe on the affected host exited with status `0` and
wrote nothing to stdout or stderr.
 This places the fault in FFF's native grep
implementation or its input/state,
 not in the terminal notification subprocess.

Related FFF reports are [issue 618][fff-618],
 [issue 422][fff-422],
 and
[issue 476][fff-476].

## Verification

Version under observation:

- `@ff-labs/pi-fff`:
   `0.9.6`
- `@ff-labs/fff-node`:
   `0.9.6`
- Pi's coredump executable:
   Node `26.4.0`
- FFF source trace:
   tag `v0.9.6`,
   commit `28321da22836b0e11da81e30f40f7a043b8f8fb4`

Captured crash harness:

```bash
coredumpctl --no-pager --since '10 minutes ago' info pi
```

Observed failing catalog:

- The coredump reported `Signal: 7 (BUS)`.
- The stack included `fff_live_grep`,
   `libfff_c.so`,
   and
  `ffi-rs.linux-x64-gnu.node`.
- The shell received terminal protocol bytes after Pi exited.

Observed passing catalog:

```bash
notify-send --app-name=Pi 'Pi auto-mode approval required' 'diagnostic notification'
```

This exited with status `0`,
 with empty stdout and stderr.
 Auto-mode's unit suite,
type check,
 and build also passed after the notification implementation.

## What we learned today

The nearest published reproducer was useful as a differential check,
 not as a
patch for this crash.
 At FFF parent commit
`b8e16d884bbef3625db6de13fb02f8a4275ed26d`,
 the issue 618 test reproduced:

```text
unsafe precondition(s) violated: slice::from_raw_parts requires the pointer to be aligned and non-null
process didn't exit successfully ... (signal: 6, SIGABRT)
```

The already-published fix in commit
`8e8b09f2d37882cf48254f8287ab4bf663abecaf` made that test pass,
 and the patched
checkout passed all 93 `fff-search` library tests.
 The installed FFF 0.9.6
source already contains that fix.

Therefore that patch does not explain or fix the reported coredump.
 The exact
query and fixture are still missing,
 and no applicable patch was found today.
The nearest reproducer can be rerun in an FFF checkout with:

```bash
cargo test --package fff-search --test grep_overflow_path_constraint_segfault
```

The prototype ran this command in a container limited to 2 GiB of memory and two
CPUs.
 Its disposable checkout and patch were deleted after the differential
check.

## Verified workarounds

- Temporarily remove `npm:@ff-labs/pi-fff` from the Pi package list and restart
  Pi.
   This removes the native FFF crash surface,
   but also removes `ffgrep`,
  `fffind`,
   and FFF-backed `@` completion.
- Use Pi's built-in grep and find tools instead of FFF while the native crash is
  investigated.
   This trades FFF's indexing and fuzzy behavior for non-native
  tools.
- After a crash,
   run `stty sane` or `reset` in the affected shell to restore
  terminal input handling.
   This repairs terminal state only;
   it does not prevent
  the FFF crash.

## What does not work

- Changing the `notify-send` title or action argument does not address the
  coredump's native `fff_live_grep` stack.
- Applying the issue 618 fix again does not address this report because FFF 0.9.6
  already contains it.
- Treating the shell's `pi2;1:3u5;1:3u` text as a second command failure misses
  the primary event.
   Those bytes are terminal protocol residue after Pi exited.

## Upstream filing decision

No new upstream issue is drafted.
 The duplicate search found related FFF issues
422,
 476,
 and 618,
 including completed fixes for earlier native crash variants.
The current coredump lacks the FFF query and project fixture needed to identify a
new defect,
 so another report would be non-additive.

The six-constraint check is:

1.  **Upstream fault:**
     yes,
     the coredump enters FFF's native library.
2.  **Upstream can fix it:**
     yes,
     FFF has landed fixes for related native crashes.
3.  **Supported use case:**
     yes,
     FFF explicitly provides Pi's `ffgrep` tool.
4.  **Contribution welcome:**
     no `CONTRIBUTING.md` was present in the checked FFF
    tag,
     and no prohibition on external reports was found in its README.
5.  **Likely to fix it:**
     yes,
     the related issues were triaged and closed with
    fixes.
6.  **Minimal fix prototyped:**
     no applicable fix for this coredump.
     The issue 618
    fix was verified only as a differential check,
     and FFF 0.9.6 already contains
    it.
     The reported crash still needs its query and fixture before selecting a
    patch.

[fff-422]: https://github.com/dmtrKovalenko/fff/issues/422
[fff-476]: https://github.com/dmtrKovalenko/fff/issues/476
[fff-618]: https://github.com/dmtrKovalenko/fff/issues/618

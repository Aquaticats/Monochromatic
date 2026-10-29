# Terminal PTY dependency

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Decision

Use `portable-pty` for `package/terminal/app` PTY process management.

The package needs to spawn the user's shell inside a pseudoterminal,
 read child output on a worker thread,
 write user
keystrokes back to the PTY,
 and resize the PTY when the Slint viewport changes.
`portable-pty` supplies those operations through safe trait APIs:
`PtySize`,
 `PtySystem::openpty`,
 `SlavePty::spawn_command`,
 `MasterPty::try_clone_reader`,
`MasterPty::take_writer`,
 and `MasterPty::resize`.
The source checked for this decision is `/tmp/agent/wezterm-portable-pty-20260602/pty/src/lib.rs`.

## Rejected alternatives

- `nix::pty`:
   rejected because `forkpty` is unsafe and `openpty` leaves controlling-terminal process setup to package code.
  That adds fragile Unix-specific process setup to a prototype that only needs shell spawning.
- `rustix` or direct `libc`:
   rejected because they expose lower-level POSIX PTY operations rather than a complete spawn,
  reader,
   writer,
   and resize abstraction.
- `pty-process`:
   rejected because the ready-made API is useful,
   but `portable-pty` is maintained as part of WezTerm and
  directly matches terminal-emulator PTY needs.

## Consequence

The terminal remains a Linux desktop prototype for this package,
 but the PTY abstraction does not bake raw `forkpty` or
`ioctl` calls into the app.
 Future work can keep terminal ownership on the Slint UI thread while the PTY reader thread
only forwards byte chunks into the event loop.

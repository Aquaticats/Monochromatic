# Rootless podman rejects the host D-Bus session bus (zbus SASL EXTERNAL), breaking Slint dark-mode and the rfd file picker

When music-player runs inside rootless podman against the host D-Bus session
bus,
 zbus 5.16.0's SASL EXTERNAL handshake is rejected,
 so Slint's winit
color-scheme watcher cannot read the system theme (window stays light) and
rfd's portal file picker cannot open.
 Mapping the host uid into the container
with `--userns=keep-id` fixes both.

## Symptom

Launching the GUI through the container (`mise run //package/music-player/desktop-app:run`)
prints,
 on stderr:

```text
Error watching for xdg color schemes: D-Bus handshake failed: EXTERNAL rejected by the server. Accepted mechanisms: [EXTERNAL]
```

The window then renders in light theme regardless of the host's dark
preference.
 The Open button's folder picker (also a portal call) silently
fails to appear for the same reason,
 because it shares the same session-bus
client.

A separate,
 benign line often precedes it and is unrelated:

```text
amdgpu: os_same_file_description couldn't determine if two DRM fds reference the same file description.
```

That one is a Mesa warning about the passed-through DRI render node and does
not affect rendering.

## Root cause

The D-Bus session bus authenticates with the SASL EXTERNAL mechanism:
 the
client asserts a uid,
 and the daemon validates it against the connecting
socket's peer credential (`SO_PEERCRED`).
 zbus asserts the process's effective
uid.

zbus 5.16.0,
 `src/connection/handshake/mod.rs:99`:

```rust
fn sasl_auth_id() -> Result<String> {
    let id = {
        #[cfg(unix)]
        {
            geteuid().as_raw().to_string()
        }
        // ...
    };
    Ok(id)
}
```

zbus 5.16.0,
 `src/connection/handshake/client.rs:40` and `:92`,
 where that id
becomes the EXTERNAL credential and a rejection becomes the error:

```rust
None => sasl_auth_id(),
// ...
AuthMechanism::External => Command::Auth(Some(mechanism), Some(user_id?.into_bytes())),
// ...
Command::Rejected(accepted) => {
    // returns Error::Handshake("... rejected by the server ...")
}
```

Rootless podman runs the container as uid 0 by default.
 From the host kernel,
where the daemon lives (outside the container user namespace),
 that process's
peer credential is the mapped real uid (here 1000).
 So zbus asserts uid 0
while the daemon sees 1000;
 the mismatch yields `REJECTED EXTERNAL`.

Both failing features reach this same code path:

- Slint color-scheme watcher:
   `i-slint-backend-winit-1.16.1/xdg_color_scheme.rs:26`
  opens the session bus with `let connection = zbus::Connection::session().await?;`.
- rfd folder picker:
   `rfd-0.15.4/src/backend/xdg_desktop_portal.rs:9` uses
  `ashpd::desktop::file_chooser::OpenFileRequest`;
   ashpd 0.11.1 opens the
  client connection in `src/proxy.rs:40` with `let cnx = zbus::Connection::session().await?;`.

An earlier hypothesis blamed the socket itself.
 That was wrong:
 a direct read
of the same setting through GLib's gdbus succeeds even as container uid 0 (see
Verification),
 so the socket is reachable and the daemon does answer.
 The
failure is specific to a client that asserts its uid in EXTERNAL,
 which zbus
does and GLib (empirically) does not.

## Verification

Versions under test (crates.
io,
 checksums from `Cargo.lock`):

- zbus 5.16.0,
   checksum `eee682d202a77e4a9f3b2c2bdf48a7b28af5c08c34ddf66f98c93e5e39464285`.
- ashpd 0.11.1,
   checksum `d2f3f79755c74fd155000314eb349864caa787c6592eace6c6882dad873d9c39`.
- rfd 0.15.4,
   checksum `ef2bee61e6cffa4635c72d7d81a84294e28f0930db0ddcb0f66d10244674ebed`.
- i-slint-backend-winit 1.16.1,
   checksum `f4df6ec88d452e3c22e418d20c96bc1ccbc9509e81e6596426d3388bf226a5f9`.

Host:
 Fedora,
 rootless podman,
 host uid 1000,
 session bus socket at
`$XDG_RUNTIME_DIR/bus` (mode `srw-rw-rw-`,
 owned by the host user).

This minimal Python harness reproduces zbus's behaviour exactly:
 send a nul
byte,
 then `AUTH EXTERNAL <hex of getuid()>`,
 and read the daemon's reply.

```python
# dbus-external-test.py
import socket, os
uid = str(os.getuid())
hexuid = uid.encode().hex()
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect("/run/host-bus")
s.sendall(b"\0")
s.sendall(b"AUTH EXTERNAL " + hexuid.encode() + b"\r\n")
print(f"asserted uid={uid} -> {s.recv(1024)!r}")
```

Failing case (default rootless,
 asserts uid 0):

```bash
podman run --rm --security-opt label=disable \
  -v "$XDG_RUNTIME_DIR/bus:/run/host-bus" \
  -v ./dbus-external-test.py:/test.py:ro \
  localhost/monochromatic/music-player python3 /test.py
# asserted uid=0 -> b'REJECTED EXTERNAL\r\n'
```

Working case (`--userns=keep-id`,
 asserts uid 1000):

```bash
podman run --rm --userns=keep-id --security-opt label=disable \
  -v "$XDG_RUNTIME_DIR/bus:/run/host-bus" \
  -v ./dbus-external-test.py:/test.py:ro \
  localhost/monochromatic/music-player python3 /test.py
# asserted uid=1000 -> b'OK <guid>\r\n'
```

End-to-end portal read (the exact setting Slint queries;
 `uint32 1` is
prefer-dark) succeeds under keep-id:

```bash
podman run --rm --userns=keep-id --security-opt label=disable \
  -v "$XDG_RUNTIME_DIR/bus:/run/host-bus" -e XDG_RUNTIME_DIR=/run \
  -e DBUS_SESSION_BUS_ADDRESS=unix:path=/run/host-bus \
  localhost/monochromatic/music-player \
  gdbus call --session --dest org.freedesktop.portal.Desktop \
  --object-path /org/freedesktop/portal/desktop \
  --method org.freedesktop.portal.Settings.Read org.freedesktop.appearance color-scheme
# (<<uint32 1>>,)
```

Diagnostic contrast:
 the same gdbus call also returns `(<<uint32 1>>,)` under
plain rootless (uid 0).
 GLib's EXTERNAL path is not rejected where zbus's is.
GLib's internal credential handling was not traced to source here;
 the claim
is limited to the observed behaviour difference.

In-container uid under each mapping (no D-Bus needed):

```bash
podman run --rm localhost/monochromatic/music-player id
# uid=0(root) ...
podman run --rm --userns=keep-id localhost/monochromatic/music-player id
# uid=1000(user) ...
```

## Verified workarounds

Add `--userns=keep-id` to the `podman run` invocation in the `run` task
(`package/music-player/desktop-app/mise.toml`).
 The host uid maps identically
into the container,
 so zbus asserts 1000,
 matching the peer credential,
 and the
handshake returns `OK`.

Tradeoffs:

- The container process runs as uid 1000,
   not root.
   Verified that the bound
  `/work` tree and the `music-player-cargo` named volume stay owned by the host
  uid and remain readable and writable,
   so the build cache is not invalidated
  (default rootless container-root already mapped to host 1000,
   so on-disk
  ownership is unchanged across the two mappings).
- Applied only to the `run` task.
   The `build`,
   `test`,
   `lint`,
   and
  `lint:clippy` tasks touch no D-Bus and keep the default mapping.
- keep-id leaves no `/etc/passwd` entry for uid 1000;
   tools that demand a
  username can warn.
   cargo does not,
   because `CARGO_HOME=/cargo` is set
  explicitly in the `Containerfile`.

## What does not work

- Mounting the socket without `--security-opt label=disable`:
   SELinux denies
  access to the bind-mounted socket before any auth,
   surfacing as
  `Could not connect: Permission denied` rather than the EXTERNAL rejection.
  The real `run` task already disables labelling,
   so this is only a
  test-harness pitfall.
- Forcing dark theme through an environment variable:
   there is no Slint or
  winit env override for the color scheme,
   and it would not fix the file
  picker,
   which needs a working session bus regardless.
- Relying on gdbus working as proof the app will work:
   gdbus and zbus
  negotiate EXTERNAL differently,
   so gdbus success does not predict zbus
  success.

## Why we do not file this upstream

The 5-constraint check,
 walked explicitly:

1.  Is it really upstream's fault?
     No. zbus implements SASL EXTERNAL per the
    D-Bus specification:
     assert the process uid,
     let the daemon validate it
    against the peer credential.
     The mismatch is created by rootless podman's
    default user-namespace mapping (container uid 0,
     host-visible uid 1000),
     an
    environment configuration,
     not a zbus defect.
2.  Can upstream fix it?
     There is nothing to fix in zbus;
     asserting the euid is
    correct behaviour.
     "Fall back to a different mechanism on rejection" would
    weaken a security mechanism to paper over a deployment misconfiguration.
3.  Are they supporting this use case?
     The use case (host bus,
     mismatched
    userns) is a podman deployment concern,
     not a zbus feature.
4.  Will they likely fix it?
     Not applicable;
     constraint 1 already fails.
5.  Minimal fix prototyped?
     Not applicable;
     the fix is ours (the run-task flag),
    not upstream's.

Constraint 1 fails,
 so the auto-prototype step does not trigger and no upstream
issue is drafted.
 The actionable fix lives at our boundary,
 the `run` task,
 and
is recorded above.

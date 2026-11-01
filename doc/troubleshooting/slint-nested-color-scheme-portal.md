# Slint 1.18.1 nested Wayland clients take dark and light appearance from the session bus, at startup and on change

## Symptom

Running a Slint application inside a nested Wayland compositor isolates its surface,
input,
and screenshots,
but does not isolate `Palette.color-scheme`.
Changing only the nested compositor cannot produce deterministic dark and light scenes.
Changing the host desktop environment theme does work,
but mutates unrelated user state and is forbidden for fixture verification.

Before 2026-10-05 the nested fixture could only fix the appearance at startup.
A hosted client kept its startup scheme for the whole session,
so "appearance follows the system when its preference changes" could not be verified
without touching host desktop settings.

## Root cause

Wayland has no general dark or light preference protocol.
Slint 1.18.1 reads desktop appearance over the session D-Bus instead of the Wayland connection.
The deciding source is `i-slint-backend-winit-1.18.1/xdg_desktop_settings.rs`,
read from the cargo registry of the IDE build volume.

### Where the value comes from

The watcher opens the session bus and addresses the Settings portal by its well-known name:

```rust
// i-slint-backend-winit-1.18.1/xdg_desktop_settings.rs:220
    let connection = zbus::Connection::session().await?;
    let settings_proxy: zbus::Proxy = zbus::proxy::Builder::new(&connection)
        .interface("org.freedesktop.portal.Settings")?
        .path("/org/freedesktop/portal/desktop")?
        .destination("org.freedesktop.portal.Desktop")?
        .build()
        .await?;
```

A nested Wayland child inherits the parent process's `DBUS_SESSION_BUS_ADDRESS` unless the fixture overrides it.
The child therefore reads the host portal even though `WAYLAND_DISPLAY` points at the nested compositor.

Initial reads call `ReadOne(namespace, key)` and fall back to `Read` only for `UnknownMethod`
(`xdg_desktop_settings.rs:183` to `197`).
The value mapping is:

```rust
// i-slint-backend-winit-1.18.1/xdg_desktop_settings.rs:98
fn xdg_color_scheme_to_slint(value: zbus::zvariant::OwnedValue) -> ColorScheme {
    match value.downcast_ref::<u32>() {
        Ok(1) => ColorScheme::Dark,
        Ok(2) => ColorScheme::Light,
        _ => ColorScheme::Unknown,
    }
}
```

The first windows wait up to `500ms` for that query (`APPEARANCE_QUERY_TIMEOUT`,
`xdg_desktop_settings.rs:17`),
so a slow or missing portal delays the first frame rather than flashing a default theme.

### What the toolkit listens for after startup

Slint subscribes to one signal,
before its initial reads,
and applies every later change from the signal's own arguments:

```rust
// i-slint-backend-winit-1.18.1/xdg_desktop_settings.rs:232
    let mut settings_stream = settings_proxy.receive_signal("SettingChanged").await?;

    read_all_settings(&settings_proxy, &cx).await;
```

```rust
// i-slint-backend-winit-1.18.1/xdg_desktop_settings.rs:239
    while let Some(message) = settings_stream.next().await {
        let Ok((namespace, key, value)) =
            message.body().deserialize::<(String, String, zbus::zvariant::OwnedValue)>()
        else {
            continue;
        };
        if let Some(setting) = SETTINGS.iter().find(|s| s.namespace == namespace && s.key == key) {
            (setting.apply)(value, &cx);
        }
    }
```

Two properties of that subscription decide whether an emitted signal is heard.
Neither failure is logged.

The body must be exactly two strings and a variant (D-Bus signature `ssv`).
A body carrying the integer directly (`ssu`) fails the tuple deserialization and is skipped by the `continue`.

The signal must come from the connection that owns `org.freedesktop.portal.Desktop`.
zbus 5.19.0 builds the match rule from the proxy's destination
and then filters every delivered message by that owner's unique name:

```rust
// zbus-5.19.0/src/proxy/mod.rs:1138
        let mut rule_builder = MatchRule::builder()
            .msg_type(Type::Signal)
            .sender(proxy.destination())?
            .path(proxy.path())?
            .interface(proxy.interface())?;
```

```rust
// zbus-5.19.0/src/proxy/mod.rs:1268
    fn filter(&mut self, msg: &Message) -> Result<bool> {
        let header = msg.header();
        let sender = header.sender();
        if sender == self.src_unique_name.as_ref() {
            return Ok(true);
        }
```

A second connection on the same bus emitting an otherwise identical signal is discarded.

The applied value reaches Slint markup through a dependency-registering property,
so bindings re-evaluate without any window event:
`apply_color_scheme_value` calls `ctx.set_color_scheme(scheme)` (`xdg_desktop_settings.rs:117`),
which sets `SlintContext::color_scheme` (`i-slint-core-1.18.1/context.rs:267`),
and `Palette.color-scheme` in the default `fluent` style reads it through
`SlintInternal.color-scheme` (`i-slint-compiler-1.18.1/widgets/fluent/color-scheme.slint:5`).

### What the portal contract says

Read on 2026-10-05 from `data/org.freedesktop.portal.Settings.xml` in `flatpak/xdg-desktop-portal`,
last changed by commit `3255665df016bcc46cb91793714310f0c1e36cba`.
That file documents version 2 of the interface.

- `org.freedesktop.appearance` `color-scheme` has type `u`:
  `0` no preference,
  `1` prefer dark,
  `2` prefer light.
  Unknown values are treated as `0`.
- The signal is `SettingChanged (s namespace, s key, v value)`,
  described as "Emitted when a setting changes."
  A request for the value already served is not a change,
  so it emits nothing.
- `ReadOne` returns the value in one variant layer.
  The deprecated `Read` is documented as actually returning two layers.

## Isolated solution

`package/nested-wayland-session/cli` supports a startup option and a runtime control command:

```sh
monochromatic-nested-wayland-session --socket /tmp/nws.sock --color-scheme dark -- app

printf 'color-scheme light\n' | nc -U /tmp/nws.sock   # => ok changed
printf 'color-scheme light\n' | nc -U /tmp/nws.sock   # => ok unchanged
```

For either explicit startup value,
the fixture:

1.  Creates a disposable socket directory under runtime temporary storage.
2.  Starts a private `dbus-daemon --session --nofork` message bus.
3.  Registers a minimal zbus `org.freedesktop.portal.Settings` service.
4.  Serves `ReadOne`,
    `Read`,
    and filtered `ReadAll` for appearance color scheme.
5.  Sets `DBUS_SESSION_BUS_ADDRESS` only on the hosted child.
6.  Kills the private bus and removes its socket directory when the nested session ends.

The `color-scheme dark|light` control command,
implemented in `package/nested-wayland-session/cli/src/appearance_portal.rs` as `AppearancePortal::set_color_scheme`:

1.  Replaces the served value first,
    so a client that re-reads when notified gets the announced value.
2.  Returns `ok unchanged` without emitting when that value was already served.
3.  Otherwise emits `SettingChanged` with body `("org.freedesktop.appearance", "color-scheme", <uint32>)`
    from the same connection that owns `org.freedesktop.portal.Desktop`,
    and returns `ok changed`.
4.  Restores the previous value and returns `err` if the emission fails,
    so a retry notifies again.

The command never opens another bus.
A session started without `--color-scheme` has no private bus,
and the command answers `err` naming that option instead of using the bus the compositor inherited.

Unsupported individual settings return a D-Bus error rather than inventing host font,
cursor,
or accent values.
Normal shutdown removes the temporary socket directory;
`SIGKILL` bypasses cleanup and can leave the PID-named directory behind.
Without `--color-scheme`,
the fixture does not replace the child's session bus.

## Verification

Versions under test,
from `package/ide/app/Cargo.lock` and `package/nested-wayland-session/cli/Cargo.lock`:

- `i-slint-backend-winit` 1.18.1,
  checksum `dae4fcf1b19e9cb109984d2af87d2b9485388c619265bff71dd523133888bee2`.
- `zbus` 5.19.0,
  checksum `5db4be7c075cb421e4b7ee645541604239bd243ba7c357511f4ff3a74b555907`.
- `slint-viewer` 1.18.1 from the repository's `mise` tool set.
- Fixture commits `1d5fefde8` (runtime switch) and `33d0dce2b` (frame pacing used by the screenshots).

Harness:

```sh
mise run //package/nested-wayland-session/cli:lint:rust
mise run //package/nested-wayland-session/cli:lint:clippy
mise run //package/nested-wayland-session/cli:test:all
mise run //package/nested-wayland-session/cli:build
mise run //package/nested-wayland-session/cli:inspect:color-scheme
```

### Passing cases

Measured on 2026-10-05.

- `test:all` reports 56 passing tests.
  The run happens in the package's build container,
  because this host lacks the compositor's development libraries.
- `runtime_switch_changes_value_served_by_every_read_method`:
  `ReadOne`,
  `Read`,
  and `ReadAll` serve `2` after a switch to light and `1` after a switch back.
- `runtime_switch_emits_setting_changed_only_for_real_changes`:
  a second client subscribes the way Slint does (proxy addressed to the well-known name,
  `receive_signal("SettingChanged")`),
  decodes the body as `(String, String, OwnedValue)`,
  checks signature `ssv`,
  and reads the value back at delivery time.
- `runtime_switch_never_contacts_environment_session_bus`:
  the test re-executes itself with `DBUS_SESSION_BUS_ADDRESS` naming a disposable decoy bus,
  performs a switch,
  then requires that the first connection the decoy ever reported is the test's own sentinel.
- `inspect:color-scheme` against the release binary,
  with the host session reporting `LockedHint=yes`:
  Slint reported `dark -> light -> dark`,
  `gdbus monitor` on the private bus recorded exactly
  `SettingChanged ('org.freedesktop.appearance', 'color-scheme', <uint32 2>)` then `<uint32 1>`,
  `gdbus call ... ReadOne` returned `(<uint32 2>,)` and `(<uint32 1>,)` after the respective switches,
  screenshots differed between dark and light and matched between the two dark states,
  and recorded frames had background luminance 28 (dark) and 250 (light).
  The decoy session bus in the compositor's own environment saw one connection,
  the task's sentinel,
  and no `SettingChanged`.

### Failing cases

- Before the implementation (commit `0dd5179fd`),
  `test:all` reported 43 passed and 7 failed:
  the three runtime portal tests,
  both control-response tests,
  and both parse tests for the new verb.
- With each guard removed in a disposable copy of the package,
  the committed tests fail as follows.
  No signal emitted:
  the emission test times out waiting for `SettingChanged`.
  Body `ssu` instead of `ssv`:
  the emission test fails to decode the body.
  Signal emitted for an unchanged value:
  the emission test receives `1` where it requires `2`.
  Signal emitted from a second connection on the private bus:
  the emission test times out,
  which is the sender filter described under "What the toolkit listens for after startup".
  Signal emitted before the served value is replaced:
  the emission test reads back the old value at delivery time.
  A `zbus::blocking::Connection::session()` call added to the switch:
  the isolation test sees a connection on the decoy bus before its sentinel.

### Local evidence

Disposable,
not tracked:
`~/temp/agent/live-theme-Z50lLh/evidence/color-scheme-release-pass` holds the screenshots,
`private-bus-monitor.log`,
`slint-seen.log`,
and `decoy-session-bus.log` of the passing run;
`~/temp/agent/live-theme-Z50lLh/results` holds one test log per removed guard.

## Verified workarounds

The runtime command is the supported path;
these are its boundaries.

- Only `dark` and `light` can be served.
  "No preference" (`0`) is not offered at startup or at runtime,
  so `ColorScheme.unknown` cannot be tested through this fixture.
- Only `color-scheme` is served.
  Slint 1.18.1 also watches `accent-color`,
  `font-name`,
  `cursor-blink`,
  and `cursor-blink-time` (`xdg_desktop_settings.rs:71` to `89`);
  those reads return a D-Bus error here and keep toolkit defaults.
- The private service's introspection data does not list the signal,
  because it is emitted with `Connection::emit_signal` rather than declared on the interface.
  Subscribers that use match rules,
  including Slint and `gdbus monitor`,
  are unaffected.
- `Read` returns one variant layer here,
  while the real portal returns two.
  Slint's `downcast_ref` accepts both;
  a client that insists on two layers would not.
  The October 16 trial investigates the resulting Zenity failure under
  "Zenity 4.0.1 fails on the private portal's deprecated Read reply".
- The private bus does not expose unrelated host services such as notifications or file-chooser portals.

## What does not work

- Emitting `SettingChanged` from any connection other than the one owning `org.freedesktop.portal.Desktop`.
  The subscriber silently drops it.
- Sending the integer without a variant wrapper.
  The subscriber silently skips it.
- Treating the toolkit's first reported value as proof that a frame exists.
  Slint applies the startup scheme before its first frame is committed;
  a screenshot taken 400 ms after the first report showed only the compositor's empty screen
  with the release binary.
  `inspect:color-scheme` therefore waits until a screenshot differs from the empty screen.
- Changing the host desktop theme.
  It works but mutates user state.

## Related quirks found while verifying

### `slint-viewer --on` handlers do not run through a shell

`slint-viewer --help` (1.18.1) says the handler "is going to be passed to the shell to be executed".
A handler of `echo $1 >> FILE` instead printed `dark >> FILE` on the viewer's standard output
and created no file.
The source splits the string into words and executes the first one directly:

```rust
// slint-ui/slint tag v1.18.1, tools/viewer/main.rs:603
fn execute_cmd(cmd: &str, callback_args: &[Value]) -> Result<()> {
    let cmd_args = shlex::split(cmd).ok_or("Could not parse the command string")?;
    let program_name = cmd_args.first().ok_or("Missing program name")?;
    let mut command = std::process::Command::new(program_name);
```

Works:
`--on scheme-seen 'echo scheme-seen $1'`,
reading the report from the viewer's standard output.
Fails:
any handler relying on redirection,
pipes,
or variable expansion other than `$1`,
`$2`,
and so on.
The tradeoff of the working form is that reports share the viewer's own standard output.

### `slint-viewer --on` handlers miss callbacks made from `init`

A scene that reports from `init => { root.report(...); }` printed nothing:
the handler did not exist yet.
`slint-viewer` creates the component,
which runs `init`,
and only then installs the `--on` handlers:

```rust
// slint-ui/slint tag v1.18.1, tools/viewer/main.rs:304
        let component = c.create()?;
        setup_instance(&component, &args.on, args.load_data.as_deref(), args.path())?;
```

```rust
// slint-ui/slint tag v1.18.1, tools/viewer/main.rs:378 (end of setup_instance)
    install_callbacks(instance, callbacks);
```

The appearance probe never noticed,
because its first report comes from `changed scheme` once the portal value arrives,
after the handlers exist.
The scale probe in `inspect:scale` (`package/nested-wayland-session/cli/mise.toml`)
reports its starting state from a one-shot 50 ms `Timer` instead;
timers run in the event loop,
after `install_callbacks`.
Its tradeoff:
the timer can fire before the first configure,
so the first size report may be `0 0`,
and the task only counts size reports that arrive after its first settled check.

### The private bus attempted D-Bus activation from host service files (closed 2026-10-05)

Until 2026-10-05 the private daemon ran as `dbus-daemon --session`,
which loads the stock `/usr/share/dbus-1/session.conf`
with `<standard_session_servicedirs/>` and its includes.
When `slint-viewer` asked for the accessibility bus,
the private daemon logged:

```text
Activating service name='org.a11y.Bus' requested by ':1.6' (... comm="slint-viewer ...")
Activated service 'org.a11y.Bus' failed: Failed to execute program org.a11y.Bus: Permission denied
```

The same two lines appeared again on 2026-10-05
when `inspect:scale` first ran against the release binary built before the fix.
Nothing reached the host session bus,
but a hosted client could ask the private daemon to start any service the host has a service file for.
`man dbus-daemon` (1.16.2) says `--session` is equivalent to
`--config-file=/usr/share/dbus-1/session.conf`,
and lists `<standard_session_servicedirs/>` as
`$XDG_RUNTIME_DIR/dbus-1/services`,
`$XDG_DATA_HOME/dbus-1/services`,
and `dbus-1/services` under each `XDG_DATA_DIRS` entry.

The private daemon now starts with `--config-file` pointing at a file written beside its socket
(`PRIVATE_BUS_CONFIG` in `package/nested-wayland-session/cli/src/appearance_portal.rs`):
the stock session elements that make it a session bus
(`<type>session</type>`,
`<keep_umask/>`,
`<auth>EXTERNAL</auth>`,
the allow-all default policy)
and no `<servicedir>`,
no `<standard_session_servicedirs/>`,
no `<include>` or `<includedir>`.
Measured with `dbus-send` against both configurations on 2026-10-05:

```text
stock   ListActivatableNames: 73 names, including org.a11y.Bus
stock   StartServiceByName org.a11y.Bus: org.freedesktop.DBus.Error.Spawn.ExecFailed:
        Failed to execute program org.a11y.Bus: Permission denied
private ListActivatableNames: org.freedesktop.DBus only
private StartServiceByName org.a11y.Bus: org.freedesktop.DBus.Error.ServiceUnknown:
        The name org.a11y.Bus was not provided by any .service files
```

The private daemon logs nothing for the refused request,
and neither `slint-viewer` nor the IDE printed an accessibility warning in the 2026-10-05 runs.

`dbus-daemon` 1.16.2 refuses a configuration without `<listen>`,
even when `--address` is given:
`Failed to start message bus: Configuration file needs one or more <listen> elements giving addresses`.
The file therefore lists `unix:path=/dev/null/replaced-by-the-address-option`,
a path that cannot be bound,
and the existing `--address` option overrides it,
as the manual says it does.
No path is interpolated into the file,
so it needs no XML escaping.

The guard is `private_bus_never_reads_service_files` in
`package/nested-wayland-session/cli/src/appearance_portal_tests.rs`.
It re-executes itself with `XDG_DATA_HOME` and `XDG_DATA_DIRS` naming a disposable directory
that holds one planted `.service` file,
so it does not depend on which services the machine has installed.
The first version of the test only asked for `org.a11y.Bus`;
it passed against the stock configuration inside the build container,
which has no such service file,
so it could not detect the bug and was replaced.
The planted version failed against the stock configuration
(`left: ["org.freedesktop.DBus", "org.example.MonochromaticPlantedService"]`)
and passes with the private configuration.

## Zenity 4.0.1 fails on the private portal's deprecated Read reply

### Observed failure

The Depot desktop chooser diagnostic run `38832874391`,
job `116755346132`,
recorded Zenity `4.0.1-1build3` exiting with `SIGSEGV` after:

```text
# Zenity stderr captured by the desktop chooser diagnostic
GLib-CRITICAL: the GVariant format string 'v' has a type of 'v' but the given value has a type of 'u'
GLib-CRITICAL: g_variant_get: assertion 'valid_format_string (format_string, TRUE, value)' failed
```

The subprocess was present and launched;
the missing chooser window was not evidence that Zenity was absent.
The existing failure is tracked in [repository issue 1004](https://github.com/Aquaticats/Monochromatic/issues/1004).
[Chooser diagnostic run](https://github.com/Aquaticats/Monochromatic/actions/runs/38832874391)

### Reply contract and test gap

The [current XDG Settings specification](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html#org-freedesktop-portal-settings-read)
still documents two variant layers for deprecated `Read`,
and one for `ReadOne`.
Before the trial correction,
`package/nested-wayland-session/cli/src/appearance_portal.rs:177`
delegated directly to the same setting value used by `ReadOne`:

```rust
// package/nested-wayland-session/cli/src/appearance_portal.rs, before the trial correction
fn read(&self, namespace: &str, key: &str) -> zbus::fdo::Result<OwnedValue> {
    return self.read_setting(namespace, key);
}
```

The existing tests converted each decoded reply directly to an integer or RGB tuple.
Installed `zvariant` `5.14.0`,
checksum `b5e28c25bd8bb8da5a1f3e7065d0c156b9ee9a7973adf78b0e35eaefdf3b1b5c`,
accepts both shapes in `src/value.rs:490`,
inside `Value::downcast_ref`:

```rust
// zvariant-5.14.0/src/value.rs:495
if let Value::Value(v) = self {
    <T>::try_from(v)
} else {
    <T>::try_from(self)
}
```

This automatic unwrapping hid the protocol mismatch from value-only assertions.
`Value::value_signature` preserves the distinction in `src/value.rs:296`:

```rust
// zvariant-5.14.0/src/value.rs:296
Value::Value(_) => &Signature::Variant,
```

The trial test commit `5eb02af048` checks each decoded value's contained signature before conversion:
`ReadOne` and `ReadAll` require `u` for color scheme or `(ddd)` for accent;
deprecated `Read` requires `v`.
The typed conversion then rejects further extra wrappers.
The calls traverse the real disposable private bus,
so they verify the serialized reply rather than a helper's in-memory return value.

### Verification status

The committed regression ran against the unchanged implementation using:

```sh
# trial/depot-20261015 worktree
mise run //package/nested-wayland-session/cli:test:all
```

The bounded package container ran 270 tests:
266 passed and four failed.
The failures assert deprecated `Read` contained signatures of `u` or `(ddd)` instead of `v`:
the accent-only portal test and the color-scheme tests for served values,
notification-time readback,
and environment-bus isolation.
The command exited with status 1,
and nextest reported no skipped tests.
Trial implementation commit `dcaae7c673` adds the deprecated method's extra wrapper.
The identical command then passed all 270 tests with no skipped tests.
After the conversion-error callback was named in trial commit `294b0db534`,
the same tests passed again,
and `mise run //package/nested-wayland-session/cli:lint:clippy` passed.
The final comment-only trial commit `d6deca5fa5` also passed scoped repository lint
for `appearance_portal.rs`,
`appearance_portal_tests.rs`,
and `appearance_accent_tests.rs`.
That check retained three pre-existing `rust/no-anonymous-functions` warnings and reported no errors.
The [complete cloud rerun](https://github.com/Aquaticats/Monochromatic/actions/runs/38833901553/job/116758578135)
at immutable source `93560fbb09` then passed 117 checks with no failures and one not-injectable check.
All seven sessions exited with status 0.
The chooser check observed a dialog covering 800000 pixels,
closed it with Escape,
and verified keyboard focus returned to the player by opening Search with `Ctrl+F`.
The subprocess probe recorded Zenity exiting with status 1 and no signal after cancellation;
its captured stderr contained no variant-format assertion or `SIGSEGV`.
This verifies the correction at the original chooser boundary on the prepared Depot runner.
The job took 410 seconds,
including a 185-second interaction step.

The same captured stderr retains separate
[accessibility-bus](https://github.com/Aquaticats/Monochromatic/issues/1041),
[dconf-service](https://github.com/Aquaticats/Monochromatic/issues/1042),
and [transient-parent](https://github.com/Aquaticats/Monochromatic/issues/1043) diagnostics.
Those diagnostics did not fail this suite;
their underlying behavior remains separate from the corrected reply contract.
The not-injectable case is `double-shift/repeat`,
whose harness explicitly reports a skip and points to pure detector coverage.
These remaining findings are tracked separately from issue 1004.
Issue 1004 remains open until the trial correction is integrated into the main branch.

### Upstream filing decision for the malformed reply

1.  The observed contract violation is in this repository's private portal,
    not an established defect in zvariant or the portal specification.
2.  The repository controls the reply wrapper and can correct it.
3.  The deprecated method's nested variant is explicitly documented.
4.  No external contribution policy is needed for the repository-owned correction.
5.  No upstream maintainer decision is being requested.
6.  The committed regression failed before the wrapper correction and passed afterward;
    the complete cloud rerun also passed the chooser open,
    cancel,
    and restored-focus checks.

No upstream issue or comment is proposed.
The existing repository issue owns the consumer failure.

## Upstream filing decision

`.out-of-scope/` was inspected and has no Slint or xdg-desktop-portal entry.

### Runtime appearance switching

Nothing is filed.
The gap was in this repository's fixture,
which did not serve a changeable value.
Slint,
zbus,
and the portal behaved as their sources and documentation state.

### `slint-viewer --on` help text

1.  Upstream's fault:
    yes.
    The help text promises a shell;
    `tools/viewer/main.rs:603` to `606` execute the program directly.
    The same text and code are on the default branch as of 2026-10-05 (lines `141`,
    `607`,
    and `609`).
2.  Fixable:
    yes,
    by correcting one doc comment or by invoking a shell.
3.  Supported use case:
    yes,
    `--on` is a documented option.
4.  Contribution welcome:
    `CONTRIBUTING.md` says "Please feel welcome to open GitHub issues or pull requests"
    and refers AI coding assistants to `AGENTS.md`.
    No ban on assisted reports was found in `CONTRIBUTING.md` or `AGENTS.md`.
5.  Likely to be fixed:
    no contrary signal.
    `gh search issues --repo slint-ui/slint` for `viewer --on shell`,
    `slint-viewer callback handler shell`,
    and `viewer shlex`,
    and `gh search prs` for `viewer --on handler`,
    returned no results.
6.  Prototype:
    the documentation-only change is written out here.
    It was not compiled,
    because it alters no behavior and building `slint-viewer` from source was outside this work.

```diff
--- a/tools/viewer/main.rs
+++ b/tools/viewer/main.rs
@@ -138,5 +138,5 @@
     /// Specify callbacks handler.
-    /// The first argument is the callback name, and the second argument is a string that is going
-    /// to be passed to the shell to be executed. Occurrences of `$1` will be replaced by the first argument,
-    /// and so on.
+    /// The first argument is the callback name, and the second argument is a command line. It is split
+    /// into words with shell quoting rules and executed directly, without a shell, so redirection and
+    /// pipes are not available. Occurrences of `$1` will be replaced by the first argument, and so on.
     #[arg(long, value_names(&["callback", "handler"]), number_of_values = 2, action)]
```

Do not file as-is:
the diff is uncompiled and filing upstream is a decision for the repository owner.

~~~md
Title: slint-viewer: `--on` help says the handler is passed to the shell, but it is executed directly

Version: slint-viewer 1.18.1 (also present on master as of 2026-10-05).

`slint-viewer --help` documents `--on <callback> <handler>` as
"a string that is going to be passed to the shell to be executed".

`tools/viewer/main.rs` `execute_cmd` splits the string with `shlex::split`
and runs the first word with `std::process::Command::new`, so no shell is involved.

Reproduction:

```sh
slint-viewer --on report 'echo $1 >> /tmp/report.log' scene.slint
```

with a scene that calls `report("dark")`.
Expected from the help text: `/tmp/report.log` contains `dark`.
Actual: `dark >> /tmp/report.log` is printed on the viewer's standard output and no file is created.

Suggested fix: correct the doc comment on the `on` argument to say the command line is split
into words and executed without a shell (or run it through `sh -c` if a shell was intended).

This report was prepared with AI assistance; the reproduction and source reading were checked by running
the installed 1.18.1 binary and reading the tagged source.
~~~

### `slint-viewer --on` and callbacks made from `init`

Nothing is filed.

1.  Upstream's fault:
    no.
    The `--on` help text promises a handler for a callback,
    not delivery of calls made while the component is constructed.
    The interpreter can only accept a handler on an instance that exists,
    and creating it runs `init` (`tools/viewer/main.rs:304`,
    then `:378`).
2.  Fixable:
    only by changing how the viewer constructs components,
    for example by queuing early calls;
    no evidence was gathered that this is wanted.
3.  Supported use case:
    not documented either way.
4.  Contribution welcome:
    as for the help-text item in this section.
5.  Likely to be fixed:
    `gh search issues --repo slint-ui/slint` for `viewer --on init`
    and `slint-viewer callback init handler` returned no results on 2026-10-05.
6.  Prototype:
    none,
    because constraint 1 fails.

The consumer-side workaround,
a one-shot `Timer`,
is recorded under "Related quirks found while verifying".

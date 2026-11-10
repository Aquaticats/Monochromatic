# rfd 0.18.0's zenity backend discards the chooser's standard error, so a failed folder dialog writes nothing and looks like a cancel

`rfd`'s Linux folder picker answers `Option<PathBuf>`.
 A cancel and a `zenity` that could not open a window both become `None`,
 and the program's own standard error,
 the one line that
says why,
is captured and thrown away.
 The application that shipped until issue 1010 also installed a `tracing` subscriber without the `log` bridge,
 so the one
record the library can emit about a failure went nowhere either.

## Symptom

Clicking `Open folder` (or pressing the shortcut that opens the same dialog) does nothing.
 No window appears,
 no error
message appears,
 and the application log records only that the chooser was asked for:

```text
2026-10-16T01:00:26.962672Z  INFO music_player::dialog_backend: folder chooser backends zenity=true portal=true
```

That pair of probes is the last thing either backend says.
 The retained log of the desktop trial run
(`gh run view 38832349057 --repo Aquaticats/Monochromatic --log`,
job `desktop`,
step `Diagnose the folder chooser with retained application output`)
holds that one line and then no further record about the chooser,
although the key press that asks for the dialog was consumed
(`ui_state {"reason":"key",...}` immediately before it).

## Root cause

Three links produce the silence,
in this order.

### 1. The picker's answer has two values, not four

`rfd-0.18.0/src/backend/xdg_desktop_portal.rs:165-197` is the whole Linux folder picker:

```rust
impl FolderPickerDialogImpl for FileDialog {
    fn pick_folder(self) -> Option<PathBuf> {
        let window_identifier = to_window_identifier(self.parent, self.parent_display);
        let res = portal::open_file(portal::OpenFileOptions { /* ... */ });
        if let Some(res) = res {
            match res {
                portal::Response::Selected(res) if res.is_empty() => None,
                portal::Response::Selected(mut res) => Some(res.remove(0)),
                portal::Response::Canceled => None,
            }
        } else {
            warn!("Using zenity fallback");
            match block_on(zenity::pick_folder(&self)) {
                Ok(res) => res,
                Err(err) => {
                    error!("Failed to pick folder with zenity: {err}");
                    None
                }
            }
        }
    }
```

A portal that answers `Canceled` and a portal call that fails and falls through to `zenity` both end in `None`,
and so
does a `zenity` run that reports an error.
 The caller cannot tell these apart,
 because the type says they are the same.

Source:
[rfd `src/backend/xdg_desktop_portal.rs` at the 0.18.0 tag](https://github.com/PolyMeilex/rfd/blob/2d35d7d07ab4d021ca92e5378ae0e6ed5fbf8d01/src/backend/xdg_desktop_portal.rs#L165-L197).

### 2. The zenity backend keeps only standard output

`rfd-0.18.0/src/backend/linux/zenity.rs:76-87` runs the program through `Command::output`,
which captures both streams,
and then uses `stdout` alone:

```rust
async fn run(mut command: Command) -> ZenityResult<Option<String>> {
    let res = {
        let (tx, rx) = crate::oneshot::channel();
        std::thread::spawn(move || {
            tx.send(command.output()).ok();
        });
        rx.await.map_err(std::io::Error::other)??
    };
    let buffer = String::from_utf8(res.stdout)?;

    Ok((res.status.success() || !buffer.is_empty()).then_some(buffer))
}
```

`res.stderr` is never read.
 A `zenity` that exits 1 and prints nothing on standard output reaches line 86 with
`status.success() == false` and an empty buffer,
so the function returns `Ok(None)`:
an exit failure is reported to the caller as
"the user chose nothing".
 The same shape is in `rfd-0.17.2`,
 the release the issue cites (`src/backend/linux/zenity.rs:67-78`),
 so the mechanism predates the 0.18.0 the crate resolves to today.

Source:
[rfd `src/backend/linux/zenity.rs` at the 0.18.0 tag](https://github.com/PolyMeilex/rfd/blob/2d35d7d07ab4d021ca92e5378ae0e6ed5fbf8d01/src/backend/linux/zenity.rs#L76-L87).

### 3. The one record the library can emit went to an unbridged facade

`rfd` writes those two records,
`warn!("Using zenity fallback")` and `error!("Failed to pick folder with zenity: {err}")`,
through the `log` facade (`use log::{error, warn};`,
`rfd-0.18.0/src/backend/xdg_desktop_portal.rs:15`).
Until issue 1010 this application installed a `tracing` subscriber with
`tracing::subscriber::set_global_default`,
which leaves `log` unbridged,
so both records were dropped before any sink saw them.
 The bridge is now installed
(`try_init` in `package/music-player/desktop-app/src/logging.rs`),
with the ICU targets turned off so their CJK-segmentation warnings stay silent.

### What an earlier reading got wrong

The issue's title calls the two halves "unbridged log facade" and "discarded Zenity stderr" as one cause.
 They are
separate,
and only the first can be fixed by bridging:
with the bridge installed,
a `zenity` that cannot be started now reports
`Failed to pick folder with zenity: <io error>`,
but a `zenity` that starts and fails still returns `Ok(None)` with its standard error
gone.
 That is why
`package/music-player/desktop-app/src/folder_chooser.rs` runs `zenity` itself where no desktop portal answers.

## Verification

Versions under test:

- `rfd` 0.18.0,
  crates.io checksum `c4b9d4857b10297ffb0f64cae711e983a7aeaee561848d56d429d3858bd8ed93`
  (`package/music-player/desktop-app/Cargo.lock`).
  The quoted lines were re-read in the vendored source at
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rfd-0.18.0/` and in the tagged upstream file fetched with
  `gh api repos/PolyMeilex/rfd/contents/...?ref=2d35d7d07ab4d021ca92e5378ae0e6ed5fbf8d01`.
- `zenity` 4.2.2 on the host that wrote this note.

### Harness

The chooser's own streams,
with no display so that no window opens anywhere:

```bash
# package/music-player/desktop-app sources
cargo build --release
timeout 20 env -u DISPLAY -u WAYLAND_DISPLAY zenity --no-markup --file-selection --directory
```

What `rfd` makes of those streams,
in a throwaway crate outside this repository that depends on `rfd 0.18.0`
and prints every `log` record,
run with the session bus and both displays unset and `PATH` holding one stub
`zenity`:

```rust
log::set_logger(&PRINTER).expect("logger");
log::set_max_level(log::LevelFilter::Trace);
println!("PATH={:?}", std::env::var_os("PATH"));
let picked = rfd::FileDialog::new().pick_folder();
println!("pick_folder() = {picked:?}");
```

```bash
env -u DBUS_SESSION_BUS_ADDRESS -u XDG_RUNTIME_DIR -u DISPLAY -u WAYLAND_DISPLAY PATH=/probe/fake-bin ./rfd-chooser-probe
```

### Patterns that fail

A `zenity` that cannot open a display exits 1 and writes its explanation to standard error:

```text
$ timeout 20 env -u DISPLAY -u WAYLAND_DISPLAY zenity --no-markup --file-selection --directory
exit=1
stdout: (empty)
stderr: (zenity:860016): Gtk-WARNING **: 00:46:06.650: Failed to open display
```

Through `rfd` that is an empty answer with no record of the reason,
which is the whole defect:

```text
$ PATH=/probe/fake-bin ./rfd-chooser-probe        # stub: printf 'Failed to open display\n' >&2; exit 1
log[WARN] rfd::backend::xdg_desktop_portal: Using zenity fallback
pick_folder() = None
```

The library's warning says which way it tried,
and nothing says why the program exited 1.

A `zenity` that is named on the search path but cannot be started gives one record and the same empty answer:

```text
$ PATH=/probe/fake-bin ./rfd-chooser-probe        # stub: mode 0644, so exec fails
log[WARN] rfd::backend::xdg_desktop_portal: Using zenity fallback
log[ERROR] rfd::backend::xdg_desktop_portal: Failed to pick folder with zenity: Permission denied (os error 13)
pick_folder() = None
```

That `ERROR` line is the record the application never showed before the bridge existed.

### Patterns that work

A `zenity` that prints a folder and exits 0 gives `Some(path)`:

```text
$ PATH=/probe/fake-bin ./rfd-chooser-probe        # stub: printf '/a/chosen/folder\n'
log[WARN] rfd::backend::xdg_desktop_portal: Using zenity fallback
pick_folder() = Some("/a/chosen/folder")
```

A cancel exits 1 and prints nothing on either stream,
and reads exactly like the failing case above:

```text
$ PATH=/probe/fake-bin ./rfd-chooser-probe        # stub: exit 1
log[WARN] rfd::backend::xdg_desktop_portal: Using zenity fallback
pick_folder() = None
```

## Verified workarounds

Both are in `package/music-player/desktop-app` and both are covered by unit tests there.

### Run the chooser program here and classify its streams

`src/folder_chooser.rs` resolves `zenity` from the search path and runs it with
`--no-markup --file-selection --directory`,
keeping both streams.
 A folder on standard output is a choice;
 an exit failure with a non-blank standard error is a failure whose text is shown to the user;
 an exit failure with an empty standard error is a cancel.
 Tradeoff:
 a cancel that also writes something to standard error (a GTK warning,
for instance) is reported as a failure,
which is the direction that keeps the explanation rather than hiding it.

### Bridge the `log` facade and silence the ICU targets

`src/logging.rs` installs the subscriber with `try_init`,
which adds `tracing-log`'s bridge because it is one of
`tracing-subscriber`'s default features,
and prepends `icu_provider=off,icu_segmenter=off` to the filter directives.
 Tradeoff:
 other crates' `log` records now reach the sink at their own levels,
so a dependency that logs at `info`
newly appears in the application log;
and an operator has to name an ICU target in `RUST_LOG` to hear it again.

## What does not work

- Reading `rfd`'s `Option<PathBuf>` as the outcome.
  The type has no third value,
  so no caller can separate a cancel from a
  failure.
- Bridging `log` without silencing `icu_provider`.
  In a debug build that restores the CJK-segmentation flood,
  one line per
  text-layout pass (`doc/troubleshooting/slint-parley-icu-cjk-segmentation.md`).
- Probing for the program before opening the dialog and reporting "no chooser installed" on a failed run.
  `zenity` is on
  the search path in the failing case,
  so the probe passes and the dialog still never appears.

## Upstream filing decision

Nothing to file.

1. Is it really upstream's fault?
   Partly.
   Discarding `res.stderr` in `run` is upstream's choice,
   and keeping it in the error type would be a
   small change.
   Returning `Option<PathBuf>` is the API's shape,
   which is a design decision rather than a defect.
2. Can upstream fix it?
   Yes:
   `ZenityError` could carry the standard error text,
   and `run` already has it in hand.
3. Are they supporting this use case?
   The backend exists to drive `zenity`,
   and `pick_folder` is a documented entry point,
   but no doc,
   test,
   or example covers
   reporting why a chooser failed.
4. Would the repo welcome our contribution?
   Not established.
   The tagged tree has no `CONTRIBUTING.md` and no `.github/ISSUE_TEMPLATE`
   (`gh api repos/PolyMeilex/rfd/contents/` and `.../contents/.github` list `workflows` only),
   so nothing states
   whether an outside report is wanted.
5. Is the report actionable and free of our private data?
   The failure mode is,
   but the trigger needs a `zenity` that fails,
   which is
   environment-specific.
6. Would a maintainer reproduce it from the report alone?
   Only with the harness above,
   which needs a session without a display.

The application no longer depends on upstream for this diagnostic,
so a report would carry no information we still need.

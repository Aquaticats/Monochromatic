# Rio 0.5.29 reports installed JetBrains Mono as missing after selecting a variable-font instance

## Symptom

On 2026-11-01,
the Fedora package `rio-0.5.29-1.fc45.x86_64` reported `Font(s) not found:` for `JetBrains Mono`.
The family was installed in the user's font directory,
and `fc-list 'JetBrains Mono'` returned both static and variable files.
Rio's configuration requested that family for regular,
bold,
italic,
and bold italic text.

The diagnostic comes from Rio's `RioErrorType::FontsNotFound` display implementation,
[`rio-vt/src/error/mod.rs:73-80`][warning]:

```rust
// Rio v0.5.29, rio-vt/src/error/mod.rs:80
write!(f, "Font(s) not found:\n{font_str}")
```

## Root cause

Rio discovers the family but fails while parsing its selected regular face.
Its debug log selects `JetBrainsMono-Variable.ttf` with index `458752`.
The other styles select static files with index `0` and load successfully.

Fontconfig encodes both a collection face and a named variable-font instance in its index.
[`src/fcfreetype.c:2223-2225`][fontconfig-index]
separates them:

```c
/* Fontconfig, src/fcfreetype.c:2223-2225 */
FcBool         index_set = id != (unsigned int)-1;
unsigned int   set_face_num = index_set ? id & 0xFFFF : 0;
unsigned int   set_instance_num = index_set ? id >> 16 : 0;
```

For the observed `458752`,
the collection face is `0` and the named-instance component is `7`.
Rio instead passes the complete index to Swash's collection-face parser in
[`sugarloaf/src/font/mod.rs:1355-1367`][parse]:

```rust
// Rio v0.5.29, sugarloaf/src/font/mod.rs:1366-1367
let font = FontRef::from_index(&data, face_index as usize)
    .ok_or_else(|| format!("Failed to load font from path: {:?}", path))?;
```

That call rejects the standalone TTF at the supplied index.
The caller translates the parsing error into `FindResult::NotFound`
at [`sugarloaf/src/font/mod.rs:2261-2265`][failure]:

```rust
// Rio v0.5.29, sugarloaf/src/font/mod.rs:2261-2265
Err(err_message) => {
    tracing::info!(
        "Failed to load font '{query:?}', {err_message}"
    );
    return FindResult::NotFound(font_spec);
}
```

Thus the visible warning conflates an installed font's parsing failure with a missing family.
This is a separate incident from
[the Ghostty Fontconfig language-type crash](ghostty-fontconfig-language-type.md).
No common cause was established.

## Verification

The installed binary reports `rioterm 0.5.29`.
The matching upstream tag `v0.5.29` resolves to
`6dc89d10ead6d8662212c4bce513c4c8b247ca5e`.
The Fontconfig source inspected is
`0f236723a3c26bd9745e32ba2b3ee21e5f809295`.

The reproduction used a disposable copy of the user's configuration in
`${HOME}/temp/agent/rio-font-probe-20261101`.
The original configuration was preserved as `config-before-static.toml` in that directory.
Run the same font lookup in a nested window:

```sh
# doc/troubleshooting/rio-variable-font-not-found.md
systemd-run --user --scope --quiet \
  --property=CPUQuota=200% --property=MemoryMax=1G \
  --property=TasksMax=128 --property=IOWeight=20 \
  monochromatic-nested-wayland-session --size 900x600 -- \
  env RIO_CONFIG_HOME="${HOME}/temp/agent/rio-font-probe-20261101" \
  RIO_LOG_LEVEL=debug rioterm --command /usr/bin/sleep 5
```

The observed failing selection was:

```text
# Rio baseline log, timestamps and ANSI decoration removed
face_source: Path source - /home/user/.local/share/fonts/JetBrainsMono-Variable.ttf, index 458752
Failed to load font 'Query { families: [Name("JetBrains Mono")], weight: Weight(400), stretch: Normal, style: Normal }', Failed to load font from path: "/home/user/.local/share/fonts/JetBrainsMono-Variable.ttf"
```

### Inputs that fail

- The original family configuration with system discovery selects the variable regular face at index `458752`.
  Both the info-level and debug-level launches record its parsing failure.

### Inputs that load successfully

- The original configuration loads static italic,
  bold,
  and bold italic files at index `0`.
- The static-directory configuration loads `JetBrainsMono-Regular.ttf`,
  `JetBrainsMono-Bold.ttf`,
  `JetBrainsMono-Italic.ttf`,
  and `JetBrainsMono-BoldItalic.ttf`.
- A fresh launch using a copy of the applied configuration records all those success paths
  and neither `Failed to load font` nor `Failed to find font`.
  An assertion checked each expected filename and each symlink target.

The nested baseline screenshot contained a notification panel;
the static-directory screenshot did not.
Neither screenshot establishes text-rendering quality because the child command was `sleep`.
The font-loader logs are the decisive verification for this warning.

## Verified workaround

At the user's request,
the workaround was applied to `${HOME}/.config/rio/config.toml`.
Its `[fonts]` section now includes:

```toml
# ~/.config/rio/config.toml on the diagnosed host
[fonts]
# Prefer static faces to avoid Rio 0.5.29 variable-font index loading failures.
additional-dirs = ["/home/user/.config/rio/static-fonts"]
family = "JetBrains Mono"
```

`${HOME}/.config/rio/static-fonts` contains symlinks to the existing regular,
bold,
italic,
and bold italic TTF files in `${HOME}/.local/share/fonts`.
The existing per-style family settings were preserved.
No installed fonts were removed or replaced.

Tradeoff:
Rio uses the static faces for this family,
so these selections do not provide arbitrary variable-font weight interpolation.
The symlinks depend on the existing static files retaining their paths.
Other applications retain access to both the variable and static fonts.

The backup is `${HOME}/temp/agent/rio-font-probe-20261101/config-before-static.toml`.
Removing the added `additional-dirs` entry restores Rio's previous discovery behavior.
The workaround has been verified in a fresh process;
existing windows were not restarted during verification.

## What does not work

- Treating the warning as proof the family is absent:
  Fontconfig enumerates the files and Rio logs the selected variable file.
- Changing the family spelling is not supported by this evidence:
  the exact configured family already resolves and the other styles load.
- A comma-separated module filter in `[developer].log-level` did not enable logging.
  `RIO_LOG_LEVEL=info` and `RIO_LOG_LEVEL=debug` produced the useful diagnostics.
- The old missing-font issue `raphamorim/rio#1355` includes different versions,
  platforms,
  and family-name lookup failures.
  Its existence does not establish the same root cause.

## Upstream filing decision

The `.out-of-scope/` file inventory contained no Rio-specific exclusion.
Tracker searches on 2026-11-01 used `variable font not found`,
`JetBrains`,
and PR terms `font index`.
The narrow issue and PR searches returned no results;
the broader family search returned older and unrelated reports,
including issue `1355`.
This is not an exhaustive duplicate or work-in-progress audit.

1.  Upstream fault:
    the installed Rio process supplies a compound font index to a collection-face parser.
2.  Fixability:
    the boundary can distinguish face and instance indices;
    preserving the chosen variation coordinates needs separate implementation verification.
3.  Supported use case:
    this investigation exercises Rio's existing font-family configuration and installed variable-font discovery.
4.  Contribution acceptance:
    the checked README and `.github` policy text contained no AI-contribution ban.
    No filing-readiness conclusion is made from that bounded check.
5.  Maintainer intent:
    no exact matching report or fix was established by these searches.
6.  Upstream prototype:
    none was created.
    The requested action was the static configuration workaround,
    which was applied and verified without changing Rio's source.

No upstream message was sent and no filing recommendation is made.
The latest-release matrix,
trust-system audit,
full work-in-progress search,
and source-fix regression tests were not run.

[warning]: https://github.com/raphamorim/rio/blob/6dc89d10ead6d8662212c4bce513c4c8b247ca5e/rio-vt/src/error/mod.rs#L73-L80
[fontconfig-index]: https://github.com/fontconfig/fontconfig/blob/0f236723a3c26bd9745e32ba2b3ee21e5f809295/src/fcfreetype.c#L2223-L2225
[parse]: https://github.com/raphamorim/rio/blob/6dc89d10ead6d8662212c4bce513c4c8b247ca5e/sugarloaf/src/font/mod.rs#L1355-L1367
[failure]: https://github.com/raphamorim/rio/blob/6dc89d10ead6d8662212c4bce513c4c8b247ca5e/sugarloaf/src/font/mod.rs#L2261-L2265

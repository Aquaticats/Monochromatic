# Ghostty 1.3.1 crashes during Claude Code font fallback on Bazzite testing 45

## Symptom

After the 2026-11-01 reboot into Bazzite `testing-45.20261101`,
starting Claude Code caused the entire Ghostty process to exit.
The current boot's journal records Ghostty renderer-thread crashes:

- PID `9435`,
  at `18:52:42 EST`:
  `free(): invalid pointer`,
  then `SIGABRT`.
- PID `10057`,
  at `18:52:58 EST`:
  `free(): invalid pointer`,
  then `SIGABRT`.
- PID `10713`,
  at `18:53:53 EST`:
  `SIGSEGV` in `FcLangSetHasLang`.

These are native Ghostty crashes.
The core records identify `/usr/bin/ghostty --gtk-single-instance=false`.
The prior memory-pressure incident in
[the terminal-scope investigation](systemd-oomd-reaps-terminal-scope-with-the-agent-in-it.md)
does not describe these crashes.

## Root cause

Ghostty assumes Fontconfig's `lang` value is a language-set pointer.
In this reproduction it is the string `"en"`.
Ghostty passes that string's address to `FcLangSetHasLang`,
which interprets the text and adjacent memory as a language-set structure.
The resulting invalid reads or frees terminate the process.

The installed executable reports `Ghostty 1.3.1`,
`ReleaseFast`,
and the `fontconfig_freetype` backend.
Its RPM is `ghostty-1.3.1-3.fc45.x86_64`.
The loaded library is `fontconfig-2.18.3-2.fc45.x86_64`.

The checked-out Ghostty `v1.3.1` source is commit
`332b2aefc6e72d363aa93ab6ecfc86eeeeb5ed28`.
In `src/font/discovery.zig:323`,
font discovery extracts the language-set union member without checking its tag:

```zig
// Ghostty v1.3.1, src/font/discovery.zig:323-329
return DeferredFace{
    .fc = .{
        .pattern = font_pattern,
        .charset = (try font_pattern.get(.charset, 0)).char_set,
        .langset = (try font_pattern.get(.lang, 0)).lang_set,
        .variations = self.variations,
    },
```

In `src/font/DeferredFace.zig:273`,
the character lookup checks whether a fallback font supplies text or emoji presentation:

```zig
// Ghostty v1.3.1, src/font/DeferredFace.zig:273-281
if (!fc.charset.hasChar(cp)) return false;
if (p) |desired| {
    const emoji_lang = "und-zsye";
    const actual: Presentation = if (fc.langset.hasLang(emoji_lang))
        .emoji
    else
        .text;
```

The original core's renderer stack and the standalone reproduction both reach this call:

```text
# coredumpctl info 10713, current boot
FcLangSetHasLang
font.Collection.Entry.hasCodepoint
font.CodepointResolver.getIndex
font.SharedGrid.getIndex
font.shaper.run.RunIterator.next
renderer.generic.Renderer(renderer.OpenGL).rebuildCells
renderer.generic.Renderer(renderer.OpenGL).updateFrame
```

The debugger establishes the actual type confusion,
rather than inferring it from the stack:

```text
# GDB, ghostty +show-face --cp=0x23bf
# Break at FcLangSetHasLang when the second argument is "und-zsye".
x/s $rdi
0x5555573086f0: "en"
x/s $rsi
0x5555555ff622: "und-zsye"
bt 3
#0 FcLangSetHasLang () from /usr/lib64/libfontconfig.so.1
#1 font.Collection.Entry.hasCodepoint ()
#2 font.CodepointResolver.getIndex ()
continue
Program received signal SIGSEGV, Segmentation fault.
```

The architecture here is x86-64,
where these registers hold the first and second function arguments.
Claude's exact emitted byte stream was not captured.
However,
looking up `U+23BF` (`⎿`) independently reaches the same invalid call and crashes.
Claude is not required to reproduce the defect.

The reboot changed the base deployment from `44.20261026` to `testing-45.20261101`.
The retained stable deployment was not booted or tested during this investigation.
The precise Fontconfig or font-data change that first exposed the pre-existing Ghostty assumption remains unbisected.

## Verification

Run from a surviving terminal or command runner.
These probes exit or crash their own Ghostty helper process without opening a window.
Each diagnostic was capped with `systemd-run --user --scope`,
`CPUQuota=100%`,
`MemoryMax=512M`,
`TasksMax=64`,
and `IOWeight=10`.
GDB used `MemoryMax=2G`.

```bash
# Installed Ghostty, on Bazzite testing-45.20261101
ghostty +show-face --cp=65
ghostty +show-face --cp=0x23bf
```

Working catalog:

- `--cp=65`:
  exit `0`,
  `U+41 « A » found in face “JetBrains Mono”.`
- `--version`:
  exit `0`,
  reports `1.3.1`.

Failing catalog:

- `--cp=0x23bf`:
  exit `139`,
  segmentation fault.
- `--string='A⎿✻⏵😀'`:
  exit `139`.
- Original GUI incidents:
  `SIGABRT` with `free(): invalid pointer`,
  or `SIGSEGV`,
  on the renderer thread in `FcLangSetHasLang`.

Inspect the retained incident evidence:

```bash
# Journal and core metadata, read-only
journalctl --boot --no-pager _EXE=/usr/bin/ghostty --output=short-iso
coredumpctl info 10713 --no-pager
```

The raw core and debugger transcripts remain in the investigator's private scratch directory.
They are not attached to a public report because terminal process memory can contain private data.

## Verified workarounds

No runtime workaround was established.
No installed package,
Ghostty configuration,
font cache,
or deployment was changed.

Ghostty PR `14853`,
`fontconfig: handle when lang is string and not langset`,
merged on `2026-10-26T12:16:37Z` as
`8112f7676007b7823798f8a1259ab274058e8bf6`.
The fix is present in the `v1.4.0` source at
`83fb95624b0439ee660c2679b43b38c8b63769cd`,
`src/font/DeferredFace.zig:342`:

```zig
// Ghostty v1.4.0, src/font/DeferredFace.zig:342-344
const is_emoji = switch (fc.langset) {
    .lang_set => |ls| ls.hasLang(emoji_lang),
    .string => |s| std.mem.eql(u8, s, emoji_lang),
```

Updating to a package containing this fix is the source-supported remedy.
Ghostty 1.4.0 was not installed or run,
so this investigation does not claim a verified upgrade or a passing full Claude session.
Current Fedora 45 repository availability was not queried.
The older [Terra package investigation](ghostty-terra-package-version-lag.md)
describes Fedora 44 and must not be treated as a current Fedora 45 availability result.

## What does not work

- Blaming an out-of-memory kill for these incidents:
  the recorded deaths are `SIGABRT` and `SIGSEGV`,
  with the invalid-pointer call reproduced under GDB.
- Treating upstream issue `10432` as an established match:
  that thread discusses bundled and system Fontconfig symbol interposition.
  Here `readelf --dyn-syms --wide /usr/bin/ghostty` shows imported,
  undefined `FcLangSetHasLang`,
  `FcFontSort`,
  and `FcPatternGet` symbols;
  the debugger identifies a string passed as a language set.
- Explicit `--font-family` trials with `Adwaita Mono`,
  `JetBrains Mono`,
  or `monospace` returned `Unable to get font grid: error.FontconfigNoMatch`.
  That is a separate uninvestigated failure,
  not evidence that the crash is fixed.
- `--font-codepoint-map='U+23BF=Noto Sans Symbols'` still exited `139`.
- Deleting caches or rolling back the OS was not tested and is not established as a remedy.

## Upstream filing decision

Nothing to add to the existing fix;
no message was posted.
The duplicate searches were run on 2026-11-01:
Ghostty issues for `fontconfig crash`,
`FcLangSetHasLang`,
and `fontconfig 2.18`,
and pull requests for `langset`.
The last query found merged PR `14853`.
Its diff and discussion were read.

- Upstream responsibility:
  the observed invalid argument matches Ghostty's unchecked union access.
- Fixability:
  upstream has already merged the type-aware handling.
- Supported use:
  the installed GTK build and its `show-face` helper both use this lookup path.
- Contribution policy:
  `CONTRIBUTING.md` requires the project's triage and vouch process;
  no new filing is needed for this already-fixed defect.
- Maintainer direction:
  the merged fix is included in the `v1.4.0` source.
- Prototype:
  no new prototype was made;
  the existing upstream implementation was inspected,
  but its binary was not built or tested here.

Source selections were checked against local tagged clones and the GitHub API.
Browser selection verification was attempted separately;
the page operation timed out,
so no browser-verified source link is claimed.

The next action is to obtain a package containing PR `14853`,
then rerun the standalone `⎿` probe and the original Claude startup in an isolated Ghostty process.

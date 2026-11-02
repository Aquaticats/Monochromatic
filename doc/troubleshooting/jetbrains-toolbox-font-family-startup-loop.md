# JetBrains Toolbox 3.9.0 loops during startup on cached fonts without family names

## Symptom

On 2026-11-02,
`$HOME/bin/jetbrains-toolbox` remained running without showing a window.
The affected build was `3.9.0.88839`,
with bundled Skiko `0.9.37.3`,
OpenJDK `25.0.3+9-b508.4`,
and system Fontconfig `2.18.3`.

The terminal stopped printing after announcing the application log location.
The relevant file was `$HOME/.local/share/JetBrains/Toolbox/logs/toolbox.latest.log`,
not the older `toolbox.0.log`.
For PID `518894`,
line 20454 recorded:

```text
# toolbox.latest.log, 2026-11-02 13:41:43.892, UiThreadWatchdog
The UI thread 'AWT-EventQueue-0' has been blocked for 10s.
It is loading system fonts. The fontconfig cache may be stale.
Run '/usr/bin/fc-cache -fv' and restart the Toolbox App
```

Line 20507 recorded a separate notification failure:
`org.freedesktop.dbus.errors.NoReply: No reply within specified time`.
Toolbox tried to show `The Toolbox App cannot show its window`,
but the desktop notification did not reach the user.

The OpenJDK `UseCompressedClassPointers` deprecation warning,
the `java.lang.System::load` restricted-method warning,
and the IPC `Address already in use` message preceded the stall.
The same launch subsequently bound its IPC socket and entered the font-loading path.

## Root cause

The existing Fontconfig cache exposed font records without a `family` property for:

- `NotoSansCJK-wght-400-900.ttf.woff2`.
- `NotoSerifCJK-wght-400-900.ttf.woff2`.

Both files are in
`$HOME/.local/share/fonts/noto-complete/noto-cjk-main/noto-cjk-main/android`.
The full font listing contained 5,737 records,
including exactly these records with empty family output.

Skia's family enumeration loops indefinitely on this input.
The native stack captured from the original UI thread was:

```text
# GDB read-only backtrace, original Toolbox UI thread 524455
FcObjectTypeLookup.lto_priv.0
FcPatternGetString
SkFontMgr_New_FontConfig
SkFontMgrSkikoDefault
Java_org_jetbrains_skia_FontMgrKt__1nDefault
```

A breakpoint observed `FcPatternGetString` querying `"family"`
at index `468344066` and returning `1`,
`FcResultNoMatch`.
The installed binary compared the result with `3`,
`FcResultNoId`,
to exit the loop.
Other nonzero results advanced to another iteration.

The [Skia source at `src/ports/SkFontMgr_fontconfig.cpp:562`][skia-loop]
matches that disassembly:

```cpp
// JetBrains/skia, 5e54fca0283d12e0ad303d32aa87c7d13f25eef6
for (int id = 0; ; ++id) {
    FcChar8* fcFamilyName;
    FcResult result = FcPatternGetString(current, FC_FAMILY, id, &fcFamilyName);
    if (FcResultNoId == result) {
        break;
    }
    if (FcResultMatch != result) {
        continue;
    }
```

[Fontconfig `src/fcpat.c:924`][fontconfig-get],
tag `2.18.3`,
distinguishes an absent property from exhausting an existing property's values:

```c
// Fontconfig, 0f236723a3c26bd9745e32ba2b3ee21e5f809295
e = FcPatternObjectFindElt (p, object);
if (!e)
    return FcResultNoMatch;
for (l = FcPatternEltValues (e); l; l = FcValueListNext (l)) {
    if (!id) {
        *v = FcValueCanonicalize (&l->value);
        if (b)
            *b = l->binding;
        return FcResultMatch;
    }
    id--;
}
return FcResultNoId;
```

Increasing the index cannot make an absent `family` property appear.
The Java UI thread therefore stays inside native font initialization,
before creating its visible window.
The thread dump recorded 428,488.28 milliseconds of CPU time
over 431.87 seconds in this path.

The cache's creation history was not established.
A fresh cache omitted these records,
and scanning the WOFF2 files directly returned exit `1` with no font patterns.
The [prior Noto CJK WOFF2 investigation](fontconfig-noto-cjk-woff2.md)
documents rejected collections at these paths under an older Fontconfig version.
That earlier parser diagnosis does not establish how the current cache acquired incomplete records.

## Verification

The original invocation was:

```bash
# Original application invocation
"$HOME/bin/jetbrains-toolbox"
```

Read the relevant font records with:

```bash
# Fontconfig listing, without changing font files
/usr/bin/fc-list --format='%{file}\t%{family}\n'
```

Failing catalog:

- Original cache and normal Toolbox launch:
  PID `518894` remained in `FontMgrKt._nDefault`.
- `xwininfo -root -tree`,
  followed by `xwininfo -id 0x1600001`:
  Toolbox had a `30x30` input-only placeholder with `Map State: IsUnMapped`.
- Font listing before recovery:
  5,737 records,
  including the WOFF2 records without families.

Working catalog:

- A process-local Fontconfig rejection of the WOFF2 records:
  5,735 records,
  no empty families,
  and a visible Toolbox window owned by PID `1937944`.
- Fresh font cache under private scratch:
  5,735 records and no empty families without a font rejection rule.
- Rebuilt normal cache and unmodified Toolbox invocation:
  PID `2294857` created window `0x1400007`,
  titled `Toolbox`.
  `xwininfo` reported `1308x2070`,
  `InputOutput`,
  and `Map State: IsViewable`.
  `xprop` reported the matching PID and `window state: Normal`.
- The paired TTC files remained represented by 70 font records in every listing.

The recovered launch used the same Toolbox build.
No Toolbox update was needed to demonstrate recovery.
Its UI thread dump showed `WAITING (parking)` in the event queue,
instead of executing `FontMgrKt._nDefault`.
The scoped `format:monochromatic-lint` task passed,
and Pandoc's rendered HTML preserved the source excerpts,
commands,
headings,
and links.

Diagnostics used `systemd-run --user --scope` with `CPUQuota=100%`,
`MemoryMax=512M`,
`TasksMax=64`,
and `IOWeight=10`.
Font-cache rebuilding used `MemoryMax=1G`.
GDB used `MemoryMax=2G` and `TasksMax=96`.
The recovered application was left running in `toolbox-normal-recovery.scope`
with `CPUQuota=200%`,
`MemoryMax=1G`,
`TasksMax=256`,
and `IOWeight=10`.

## Verified workarounds

### Rebuild the affected directory's cache and restart Toolbox

The current cache was backed up to
`$HOME/temp/agent/toolbox-fontconfig-before-repair-20261102`.
This targeted rebuild succeeded:

```bash
# Rebuild the affected font directory's cache
font_dir="$HOME/.local/share/fonts/noto-complete/noto-cjk-main/noto-cjk-main/android"
systemd-run --user --scope --quiet \
  --property=CPUQuota=100% --property=MemoryMax=1G \
  --property=TasksMax=64 --property=IOWeight=10 \
  /usr/bin/fc-cache-64 --force --verbose "$font_dir"
```

The command reported `70 fonts, 0 dirs` for that directory and exited `0`.
After stopping the temporary filtered instance,
starting `$HOME/bin/jetbrains-toolbox` without a Fontconfig override produced the verified visible window.

This changes generated caches,
not installed fonts or Toolbox configuration.
Fontconfig also cleans invalid cache files in configured cache directories.
The rebuild logged cleanup of obsolete cache versions in `$HOME/.cache/fontconfig`.
Future regeneration could expose the upstream loop again if another familyless record appears.

The preliminary fresh-cache probe set `XDG_CACHE_HOME` to scratch,
but the normal Fontconfig configuration also names `$HOME/.fontconfig`.
Its cleanup removed obsolete cache files there.
`XDG_CACHE_HOME` alone therefore did not isolate every cache side effect.
A fully isolated future fixture must provide an explicit configuration containing only disposable cache directories.

### Exclude the affected records for one process

The temporary configuration at `$HOME/temp/agent/toolbox-font-filter.conf`
included `/etc/fonts/fonts.conf`
and rejected exact `file` patterns for the affected WOFF2 records.
Launching with `FONTCONFIG_FILE` pointing to it also restored the window.

This hides those font records from that process,
while retaining the paired TTC records.
The final recovered instance does not use this override.
The temporary configuration is retained as diagnostic evidence,
not installed as a desktop-wide font policy.

## What does not work

- Waiting for the font loop:
  the observed native loop cannot terminate on `FcResultNoMatch`.
- Treating the Java warnings as the demonstrated blocker:
  startup passed those messages and reached the independently observed Skia loop.
- Deleting IPC state as the first remedy:
  the original launch successfully rebound its socket before the UI stalled.
- Treating the D-Bus notification failure as the font-loop cause:
  it prevented the warning from reaching the desktop;
  it does not explain the native family-enumeration loop.
- Calling the cache stale based only on Toolbox's warning:
  the fresh-cache comparison and unmodified relaunch were required to establish the verified remedy.

A later debugger probe attempted to call `FcPatternFormat`
after assigning its argument to GDB's reserved `$fp` register name.
That assignment failed,
the diagnostic call used an invalid argument,
and the original process aborted.
This abort was caused by the investigation and is not evidence of an original Toolbox crash.
The loop captures predate that probe.

## Upstream filing artifact

### Upstream filing decision

No upstream message was sent.
No `.out-of-scope/` entry matched this tool or bug class.
The 2026-11-02 GitHub searches in `JetBrains/skiko`
for `fontconfig` and `font family` returned unrelated closed issues `649` and `726`.
These searches do not exhaust the general Skia or JetBrains YouTrack trackers.

- Upstream fault:
  the installed Skia loop and the inspected source mishandle an absent family property.
- Fixability:
  recognizing `FcResultNoMatch` as terminal for this enumeration is a candidate repair;
  no patch was built or tested.
- Supported use case:
  the installed Linux Toolbox invokes this Fontconfig backend during ordinary startup.
- Contribution acceptance:
  [JetBrains/skia `CONTRIBUTING:1`][skia-contributing] says
  `Only Skiko specific changes are accepted here`
  and routes general improvements upstream.
  This generic Fontconfig loop is therefore not an accepted change target in that fork.
  General Skia intake was not audited.
- Maintainer intent:
  no matching maintainer commitment or refusal was established.
- Prototype:
  none;
  the verified recovery is cache regeneration at the consumer boundary.

The source selections were loaded and checked in `agent-browser`.
Latest-release reproduction,
an upstream patch,
general Skia duplicate and work-in-progress checks,
contribution and trust-policy checks,
and a clean-runner CI reproduction were not performed.
This local draft is not ready to file.
Public filing would associate an incomplete report with the poster;
filing remains the poster's decision.

### Local draft

~~~md
<!-- Local upstream draft, not ready to file -->
Title: Fontconfig family enumeration loops when a cached pattern has no family property

JetBrains Toolbox 3.9.0.88839 with Skiko 0.9.37.3 and Fontconfig 2.18.3
stalled before creating its window.
A native breakpoint observed FcPatternGetString(pattern, "family", 468344066, ...)
returning FcResultNoMatch while SkFontMgr_New_FontConfig continued enumerating.

GetFamilyNames in SkFontMgr_fontconfig.cpp breaks on FcResultNoId
and continues on other non-match results.
Fontconfig returns FcResultNoMatch indefinitely when the family property is absent.

The local cache contained familyless records for the Noto Sans CJK and Noto Serif CJK
variable WOFF2 collections.
Excluding those records restored the window.
Rebuilding their directory's cache removed the incomplete records,
and a normal launch then worked without exclusions.

A candidate fix should terminate the enumeration when the family property is absent.
It still needs a minimal native reproduction, a tested patch, current-upstream verification,
and the owning project's intake review before submission.
~~~

[skia-loop]: https://github.com/JetBrains/skia/blob/5e54fca0283d12e0ad303d32aa87c7d13f25eef6/src/ports/SkFontMgr_fontconfig.cpp#L547-L582
[fontconfig-get]: https://github.com/fontconfig/fontconfig/blob/0f236723a3c26bd9745e32ba2b3ee21e5f809295/src/fcpat.c#L924-L943
[skia-contributing]: https://github.com/JetBrains/skia/blob/5e54fca0283d12e0ad303d32aa87c7d13f25eef6/CONTRIBUTING#L1-L3

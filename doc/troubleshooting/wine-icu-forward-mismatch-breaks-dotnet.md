# Wine 11.0 breaks .NET 6 startup when system32 icu.dll forwards to a missing icuuc68

## Symptom

Any CoreCLR (.NET) program under Wine dies at startup with this pair of
 messages,
 verbatim:

```text
0024:err:module:find_forwarded_export module not found for forward 'icuuc68.u_charsToUChars_68' used by L"C:\windows\system32\icu.dll"
Cannot get symbol u_charsToUChars from libicuuc
Error: 127
```

In the incident this killed Cpp2IL 2022.1.0-pre-release.21
 (the Il2CppAssemblyGenerator of MelonLoader 0.7.3),
 so the game's mods never loaded.
 `Cpp2IL.exe --help` reproduces it without the game.

Surface trigger:
 a wine prefix whose `drive_c/windows/system32` contains an `icu.dll`
 whose exports forward to `icuuc68.<symbol>_68`,
 while no `icuuc68` module is loadable in that Wine run.
 The same two error lines are reported elsewhere with different
 provenance
 (WineHQ bug 58661 for ILSpy and other .NET apps,
 WineHQ forum topic 39819,
 the genshin-fps-unlock issue tracker,
 and the tModLoader-on-Wine guides).

## Root cause

A wrong diagnosis recorded here so it is not re-derived:
 the host's libicu version was first blamed
 (the common explanation in the community threads:
 build-time ICU 68 versus host ICU 7x).
 In this incident that reading is wrong:
 the host ships `libicuuc.so.77` and it is irrelevant,
 because nothing ever loads it.
 The unresolvable target is a PE module named `icuuc68`,
 and the failure is prefix contamination between two Wine runtimes.

Step by step:

1.  CoreCLR initializes globalization at process start and resolves ICU
    entry points by symbol name.
    The emitting binary is
    `Microsoft.NETCore.App/6.0.36/coreclr.dll`,
    which contains the format strings
    `Cannot get symbol %s from libicuuc.`
    and `Error: %u`
    beside the probed name `u_charsToUChars%s`
    (string dump,
    command in the Verification harness).
2.  Wine resolves that probe to `system32\icu.dll`.
    In this prefix the file is byte-identical to the ICU forwarder that
    Proton ships:

```bash
$ md5sum ".../Proton - Experimental/files/lib/wine/x86_64-windows/icu.dll" \
         "/var/home/user/.local/share/wineprefixes/pvzfusion/drive_c/windows/system32/icu.dll"
4c50b6945a6b519fb9175363795fbf10  .../files/lib/wine/x86_64-windows/icu.dll
4c50b6945a6b519fb9175363795fbf10  .../system32/icu.dll
```

3.  How it got there without its targets:
    the prefix was provisioned by running Proton's `wine` binary directly
    (winetricks for `vcrun2022` and `dotnetdesktop6`),
    so the prefix update installed Proton's builtin PE dlls
    (`kernel32.dll` at 405,518 bytes sits in `system32` the same way).
    The forward targets `icuuc68.dll`,
    `icuin68.dll`,
    and `icudt68.dll` are installed only by Proton's launcher script,
    which contains

```python
                icufiles = ["icuin68", "icuuc68", "icudt68"]

                for f in icufiles:
                    self.create_symlink(self.prefix_dir + dst, g_proton.arch_pe_dir("wine/icu", False) + f + ".dll")
```

    (`proton` in the `Proton - Experimental` directory).
    That script never ran,
    so `system32` holds the forwarder without its sibling modules.
4.  Wine's forwarded-export resolution then fails to load the target
    module
    (`dlls/ntdll/loader.c:965` in tag `wine-11.0`):

```c
        if (load_dll( load_path, mod_name, 0, &wm, imp->system ) != STATUS_SUCCESS)
        {
            ERR( "module not found for forward '%s' used by %s\n",
                 forward, debugstr_w(imp->ldr.FullDllName.Buffer) );
            return NULL;
        }
```

    WineHQ Wine 11.0 provides no substitute:
    the tag has no `dlls/icu` directory at all
    (`git ls-tree wine-11.0 dlls/` shows none),
    and Proton's `files/lib/wine/icu` directory is not on this Wine's dll
    path.
5.  The symbol lookup returns nothing,
    CoreCLR prints `Cannot get symbol u_charsToUChars from libicuuc`
    plus `Error: 127`,
    and the tool exits or stalls before doing any work.
6.  The presence of a loadable-but-forward-broken `icu.dll` is worse than
    absence:
    with no `icu.dll` at all CoreCLR falls back to NLS silently,
    but a broken forwarder turns that fallback into a hard failure
    (proved in the Verification catalogs).

## Verification

Versions under test:
 WineHQ Wine 11.0 from Flathub
 (build commit `2b7feee9cd37329c1fecd0c45960b0634cd0e684a3f1fd895df8771612ed2865`),
 Proton - Experimental (Steam) as the source of the forwarder,
 Microsoft.NETCore.App 6.0.36,
 Cpp2IL 2022.1.0-pre-release.21,
 host Bazzite 44 with `libicuuc.so.77`.

Harness (cwd is the game directory;
 the game itself is not needed):

```bash
cd '/var/home/user/Games/PvZ-Fusion/Game Files'
env -u DISPLAY WINE_VIDEO_DRIVER=wayland WINEDLLOVERRIDES="version=n,b" \
  flatpak run org.winehq.Wine \
  '/var/home/user/Games/PvZ-Fusion/Game Files/MelonLoader/Dependencies/Il2CppAssemblyGenerator/Cpp2IL/Cpp2IL.exe' \
  --help
```

To confirm the emitter and the forwarder identity:

```bash
PFX="/var/home/user/.local/share/wineprefixes/pvzfusion"
grep -aob 'Cannot get symbol' \
  "$PFX/drive_c/Program Files/dotnet/shared/Microsoft.NETCore.App/6.0.36/coreclr.dll"
md5sum "/var/home/user/.local/share/Steam/steamapps/common/Proton - Experimental/files/lib/wine/x86_64-windows/icu.dll"
```

### Catalog that fails

- Default environment:
  the harness prints both error lines
  (`find_forwarded_export ... icuuc68.u_charsToUChars_68`,
  then `Cannot get symbol u_charsToUChars from libicuuc`,
  `Error: 127`)
  and exits without printing Cpp2IL usage.
- Full game launch with the same environment:
  MelonLoader's generator dies at the same point,
  so no mods load.

### Catalog that works cleanly

- Same harness with `DOTNET_SYSTEM_GLOBALIZATION_USENLS=1` added:
  Cpp2IL prints its usage banner,
  no ICU error,
  no `find_forwarded_export` line.
  Full game session with this variable:
  assembly generation completes,
  `3 Mods loaded.`
- Same harness with `WINEDLLOVERRIDES="icu=;version=n,b"` added:
  Cpp2IL prints its usage banner,
  no ICU error.
  (Disabling `icu` hides the broken forwarder and CoreCLR falls back to
  NLS on its own.)

## Verified workarounds

### Force NLS globalization for .NET

```bash
DOTNET_SYSTEM_GLOBALIZATION_USENLS=1
```

Verified in the standalone harness and in full game sessions.
 Tradeoffs:
 .NET culture,
 sorting,
 and formatting use Windows NLS data instead of ICU;
 workloads that depend on ICU-specific collation behavior would see
 differences,
 and this game tooling does not.

### Hide the incompatible forwarder from the loader

```bash
WINEDLLOVERRIDES="icu="
```

Verified in the standalone harness.
 The equivalent file-level form
 (deleting or renaming `drive_c/windows/system32/icu.dll`)
 shares the mechanism but was not applied to the live prefix.
 Tradeoffs:
 anything in that prefix that genuinely needs ICU loses it
 (it was unusable anyway while its forwards were unresolvable),
 and the override must not be used in a prefix where the `icuuc68`
 siblings do exist
 (a fully provisioned Proton prefix),
 where it would disable a working ICU.

### Keep one Wine runtime per prefix

Provision and run a prefix with the same runtime.
 For Proton that means launching through Proton's `proton` script so its
 `icufiles` symlink step runs,
 instead of invoking its `wine` binary under a different runtime later.
 Tradeoffs:
 process discipline only,
 no functional change.

## What does not work

- Installing a different Windows .NET runtime version:
  the probe that fails is CoreCLR's own ICU bootstrap,
  reproduced against the runtime installed from the winetricks cache.
- Symlinking the host ICU library to the expected versioned name
  (the workaround attempted in the Puppy Linux thread):
  ICU exports carry the version suffix
  (`u_charsToUChars_68` versus the host's `_77`),
  so a renamed `libicuuc.so` cannot satisfy the forward.
  Not retried here.
- Hoping .NET falls back to NLS while a broken `icu.dll` is loadable:
  the failing catalog disproves this.
- Reinstalling MelonLoader or the game pack:
  `Cpp2IL.exe --help` alone reproduces the failure.

## Upstream filing artifact

### Duplicate search

Searched on 2026-10-26:
 WineHQ Bugzilla
 (web search for `find_forwarded_export icuuc68`,
 `Cannot get symbol u_charsToUChars from libicuuc`),
 the wine-bugs archive mirrors,
 gitlab.winehq.org merge requests,
 and GitHub issue trackers of downstream reports.

Matching report:
 WineHQ bug 58661,
 "Visual apps, specifically with .NET won't work correctly",
 which quotes the identical `find_forwarded_export ... icuuc68`
 line and covers ILSpy and other .NET apps
 (comment 10,
 Stian Low,
 2025-09-07).
 `bugs.winehq.org` is behind the Anubis anti-bot wall as of this
 writing,
 so the thread body was read through search-engine excerpts only;
 re-read it in a browser before posting anything.

Related traffic:
 WineHQ forum topic 39819,
 genshin-fps-unlock issue 520,
 tModLoader community guides that recommend
 `DOTNET_SYSTEM_GLOBALIZATION_USENLS=1`.
 Also relevant:
 the `wine-11.6` announcement lists
 `#59553 ICU dlls cause some games to crash (Cyberpunk 2077)`
 as fixed and adds a `dlls/icu` module built against the host ICU,
 so upstream is actively reshaping this area.

### Upstream filing decision

1.  Is it really upstream's fault?
    Partly.
    The version-locked forwarder design is upstream-adjacent
    (Proton's packaging,
    and Wine master's new `dlls/icu` with
    `IMPORTS = $(ICUI18N_PE_LIBS) $(ICUCOMMON_PE_LIBS)`),
    the hard-fail-instead-of-fallback is CoreCLR's behavior,
    and this incident's trigger is mixing runtimes in one prefix.
2.  Can upstream fix it?
    Yes;
    11.6 ships movement on exactly this class.
3.  Are they supporting this use case?
    Yes;
    bug 58661 is tracked and the 11.6 fixes name .NET and ICU victims.
4.  Would the repo welcome our contribution?
    Wine development happens on gitlab.winehq.org and the wine-devel
    mailing list
    (`README.md` in the wine tree);
    no in-tree `CONTRIBUTING.md` and no AI-assisted-filing restriction
    were found in the files checked on the `wine-11.0` tag.
5.  Will they likely fix it?
    They are already changing the ICU story;
    whether 11.6 fully clears the .NET path is unverified here.
6.  Have we prototyped a minimal fix?
    No code fix is ours to prototype:
    the consumer-side workarounds are verified,
    and the upstream direction is their `dlls/icu` work.

Outcome:
 do not open a new issue,
 because duplicate bug 58661 exists.
 The additive material is the provenance analysis,
 the NLS-fallback proof,
 and the two verified workarounds,
 kept here as a comment draft.
 Mark it unfiled until the thread is re-read
 (the tracker blocked full fetch on 2026-10-26).
 No `.out-of-scope/` exemption applies
 (checked 2026-10-26:
 no wine entry).

~~~md
Comment draft for https://bugs.winehq.org/show_bug.cgi?id=58661
(do not post without re-reading the current thread)

Additional data on the `icuuc68.u_charsToUChars_68` failure:

1. Provenance variant beyond the host-ICU-version story. In a prefix
   provisioned with Proton's wine binary and later run under plain
   WineHQ Wine 11.0 (Flathub), `system32/icu.dll` is byte-identical to
   Proton's ICU-68 forwarder (md5 4c50b6945a6b519fb9175363795fbf10),
   while the sibling modules it forwards to (icuuc68, icuin68, icudt68)
   are installed only by Proton's own launcher script. The forwarder is
   therefore stranded: no icuuc68 module exists for the forward to
   resolve, regardless of which host libicu is installed.

2. .NET angle: CoreCLR's ICU bootstrap treats a loadable-but-broken
   icu.dll as a hard failure ("Cannot get symbol u_charsToUChars from
   libicuuc. Error: 127" from coreclr.dll of Microsoft.NETCore.App
   6.0.36) instead of falling back to NLS. Deleting or hiding icu.dll
   makes the same binary run, so presence of the broken forwarder is
   strictly worse than absence.

3. Verified consumer workarounds: DOTNET_SYSTEM_GLOBALIZATION_USENLS=1,
   or WINEDLLOVERRIDES="icu=" (hide the incompatible forwarder).

4. Wine 11.6 lists #59553 (ICU dlls crash Cyberpunk 2077) as fixed and
   adds dlls/icu; worth confirming whether that path also clears this
   .NET startup probe when an app-shipped or Proton-shipped icu.dll is
   present in system32.
~~~

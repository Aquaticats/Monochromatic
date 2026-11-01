# Wine 11.0 maps builtin version.dll over an app-dir doorstop proxy and silently skips mods

## Symptom

A MelonLoader-modded Unity game (PlantsVsZombiesRH.exe,
 the PvZ Fusion 4.0.5 Multi-lang pack)
 launches and renders normally,
 but no mod ever loads:
 no fresh `MelonLoader/Logs/<date>.log` appears,
 the MelonLoader console never opens,
 and the translator mod's UI stays in the game's original language.

There is no error dialog.
 `WINEDEBUG=+loaddll` shows the cause on one line:

```text
0024:trace:loaddll:build_module Loaded L"Z:\var\home\user\Games\PvZ-Fusion\Game Files\VERSION.dll" at 00006FFFFD3D0000: builtin
```

The app-directory path is printed,
 but the module is tagged `builtin`.
 With the same install and an explicit override the line becomes `native`
 and MelonLoader writes its log within seconds.

Surface trigger:
 any game or app whose loader hook is an app-dir DLL named after a DLL
 Wine also ships as a builtin.
 Doorstop-style proxies (MelonLoader,
 BepInEx,
 ASI loaders) default to `version.dll`,
 and the same shadowing applies to `winhttp.dll`,
 `dwrite.dll`,
 or `d3d9.dll` proxies whenever a matching builtin exists.

## Root cause

A wrong diagnosis recorded here so it is not re-derived:
 the MelonLoader install was first blamed
 (missing .NET runtime,
 corrupt proxy DLL).
 That reading is wrong:
 the proxy file loads fine as `native` under an explicit override,
 and the `builtin` tag on the failing trace line means the mapped image
 is Wine's,
 not the proxy's.

Step by step,
 against wine-mirror/wine tag `wine-11.0` (commit `db11d0fe6a1`):

1.  The exe's implicit import of `VERSION.dll` reaches
    `load_dll`,
    which searches and finds the app-dir file first
    (`dlls/ntdll/loader.c:3323`,
    search logged as `looking for %s in %s`).
2.  The load order comes from
    `dlls/ntdll/unix/loadorder.c`:
    `WINEDLLOVERRIDES` (line 225),
    then per-app `AppDefaults\app.exe\DllOverrides`,
    then `HKCU\Software\Wine\DllOverrides` (lines 227 to 228).
    With none set,
    the answer is `LO_DEFAULT` (`loadorder.c:437`).
3.  `load_builtin`
    (`dlls/ntdll/unix/loader.c:1370`)
    is consulted even though a native file was found;
    under `LO_DEFAULT` it asks for the builtin first
    and keeps the found native file only if no builtin exists:

```c
    switch (loadorder)
    {
    case LO_NATIVE:
    case LO_NATIVE_BUILTIN:
        return STATUS_IMAGE_ALREADY_LOADED;
    case LO_BUILTIN:
        return find_builtin_dll( nt_name, exp_name, module, size, info, limit_low, limit_high,
                                 search_machine, machine, FALSE, offset );
    default:
        status = find_builtin_dll( nt_name, exp_name, module, size, info, limit_low, limit_high,
                                   search_machine, machine, (loadorder == LO_DEFAULT), offset );
        if (status == STATUS_DLL_NOT_FOUND || status == STATUS_NOT_SUPPORTED)
            return STATUS_IMAGE_ALREADY_LOADED;
        return status;
    }
```

4.  `version.dll` has a builtin
    (`dlls/version/Makefile.in` builds `MODULE = version.dll` with no
    `-Wb,--prefer-native`),
    so `find_builtin_dll` succeeds and the native proxy is never mapped.
5.  The mapped image carries Wine's builtin signature,
    which is exactly what the trace tag reports
    (`dlls/ntdll/loader.c:2256`):

```c
    static const char builtin_signature[] = "Wine builtin DLL";
    ...
    is_builtin = ((char *)nt - signature >= sizeof(builtin_signature) &&
                  !memcmp( signature, builtin_signature, sizeof(builtin_signature) ));
```

    while `FullDllName` keeps the searched app-dir path,
    producing the confusing
    `Loaded L"...\VERSION.dll" ... : builtin` line
    (`dlls/ntdll/loader.c:2325`):

```c
    TRACE_(loaddll)( "Loaded %s at %p: %s\n", debugstr_w(wm->ldr.FullDllName.Buffer), *module,
                     is_builtin ? "builtin" : "native" );
```

6.  The proxy's `DllMain` never runs,
    so the mod loader never initializes.
    Nothing else in the launch looks broken.

## Verification

Versions under test:
 WineHQ Wine 11.0 from Flathub
 (`app/org.winehq.Wine/x86_64/stable`,
 build commit `2b7feee9cd37329c1fecd0c45960b0634cd0e684a3f1fd895df8771612ed2865`),
 source tag `wine-11.0` (`db11d0fe6a1`);
 game pack `PvZ.Fusion.Multi-lang.4.0.5.2.Alpha.by.Blooms.zip`
 (MelonLoader 0.7.3 Open-Beta,
 proxy `version.dll` 11,057,664 bytes).

Harness (cwd is the game directory,
 one run per load order):

```bash
cd '/var/home/user/Games/PvZ-Fusion/Game Files'
env -u DISPLAY WINE_VIDEO_DRIVER=wayland WINEDEBUG=+loaddll WINEDLLOVERRIDES="version=n,b" \
  flatpak run org.winehq.Wine PlantsVsZombiesRH.exe
```

Watch two outputs:
 the `build_module ... VERSION.dll` lines,
 and whether a new file appears in
 `Game Files/MelonLoader/Logs/`.

### Catalog that works cleanly

- `WINEDLLOVERRIDES="version=n,b"`:
  trace shows `VERSION.dll ... : native` for the app-dir file plus
  `C:\windows\system32\VERSION.dll ... : builtin` for the real
  implementation,
  MelonLoader logs `3 Mods loaded.`,
  full game session verified repeatedly.
- `WINEDLLOVERRIDES="version=n"` (25 s probe):
  app-dir file loads `native`,
  MelonLoader writes its log and reaches plugin loading.

### Catalog that fails

- No override (three consecutive launches):
  trace shows the app-dir path tagged `builtin`,
  no `MelonLoader/Logs` file is ever written,
  game runs unmodded.

## Verified workarounds

### Force native-first for version.dll

```bash
WINEDLLOVERRIDES="version=n,b"
```

Verified across full game sessions
 (mods load,
 translated UI appears).
 Tradeoffs:
 the native-first order applies to every `version.dll` load in that
 process,
 so a genuinely broken app-shipped `version.dll` would now win;
 keeping `,b` retains Wine's builtin as the fallback that serves the
 proxy's forwarded calls.

### Upgrade to Wine 11.6 or newer

Wine 11.6 added `prefer_native_heuristics`
 (`dlls/ntdll/unix/loadorder.c:423` in tag `wine-11.6`),
 which prefers the app-dir dll when its `CompanyName` version resource is
 present and does not start with `Microsoft`:

```c
    if (!version_find_key( &entry, companyW, &entry )) return FALSE;
    ...
    if (len >= ARRAY_SIZE(microsoftW) && !wcsnicmp( name, microsoftW, ARRAY_SIZE(microsoftW) ))
        return FALSE;
    TRACE( "preferring native from %s for %s\n", debugstr_wn( name, len ), debugstr_us( nt_name ));
    return TRUE;
```

The `wine-11.6` announcement lists
 "DLL load order heuristics to better support game mods"
 as the headline change.
 Tradeoffs:
 heuristic,
 so a proxy without a version resource,
 or one stamped `CompanyName = Microsoft`,
 still needs the override;
 not exercised here because the Flathub build under test is 11.0.

## What does not work

- Restarting or reinstalling MelonLoader:
  the failure is deterministic
  (three launches reproduced it)
  and the proxy itself is intact
  (it loads and runs under `version=n,b`).
- Suspecting the .NET runtime first:
  retried reading,
  disproved by the trace;
  the runtime and MelonLoader files were fine while the proxy never ran.
- `version=n` alone as the clean fix:
  it does load the proxy,
  but for the Wine-installed fake dll in `system32`
  `load_builtin` refuses `LO_NATIVE`
  (`dlls/ntdll/unix/loader.c:1392`,
  `if (loadorder == LO_NATIVE) return STATUS_DLL_NOT_FOUND;`),
  so the proxy's forwarded `GetFileVersionInfo*` calls have no real
  implementation to fall back to.
  The 25 s probe did not exercise a version API call,
  so this failure mode is source-derived,
  not observed.

## Upstream filing artifact

### Duplicate search

Searched Wine's trackers on 2026-10-26:
 the WineHQ Bugzilla
 (web search for `doorstop version.dll builtin`,
 `app directory native dll load order`,
 `version.dll builtin shadow`),
 the wine-bugs archive mirrors,
 and gitlab.winehq.org.
 `bugs.winehq.org` direct fetch is behind the Anubis anti-bot wall,
 so thread bodies were read through search-engine excerpts only.
 No open or closed report describes an app-dir doorstop proxy being
 shadowed by a builtin.

Nearest historical traffic:
 the 2011 wine-users thread
 "Windows has a behavior where if a DLL is located in the same directory
 as an application, windows preferentially chooses the DLL's in that
 directory",
 answered by moderator `vitamin` with
 "If Wine's dll isn't compiled to prefer native Wine will always try
 it's own first. This is intentional."
 That statement,
 plus the 11.6 heuristic,
 frames the Upstream filing decision.

### Upstream filing decision

1.  Is it really upstream's fault?
    Yes as a Windows-semantics gap
    (Windows loads the app-dir dll first),
    but the 11.0 ordering is documented policy,
    not an implementation slip.
2.  Can upstream fix it?
    Yes,
    and it did:
    `prefer_native_heuristics` in 11.6.
3.  Are they supporting this use case?
    Yes,
    explicitly:
    the 11.6 announcement names game-mod DLL loading.
4.  Would the repo welcome our contribution?
    Wine development happens on gitlab.winehq.org and the wine-devel
    mailing list
    (`README.md` in the wine tree points to both);
    no `CONTRIBUTING.md` exists in the `wine-11.0` tree and no
    AI-assisted-filing restriction was found in the tree files checked.
    Moot for filing because of item 2.
5.  Will they likely fix it?
    Already shipped in 11.6.
6.  Have we prototyped a minimal fix?
    Not needed:
    the candidate fix is present in upstream source
    (candidate-fix applicability gate:
    it is absent from the installed 11.0 build,
    and the incident reproduces on 11.0 exactly),
    and the consumer-side override already solves the user-facing problem.

Outcome:
 nothing to file.
 The fix exists upstream and the workaround covers current
 installations.
 Re-test on Wine 11.6 plus when the Flathub build moves past 11.0.
 No `.out-of-scope/` exemption applies
 (checked 2026-10-26:
 no wine entry).

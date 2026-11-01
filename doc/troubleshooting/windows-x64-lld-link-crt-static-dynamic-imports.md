# Rust nightly-2026-10-13 `x86_64-pc-windows-msvc` with `+crt-static` linked by `lld-link` (LLVM 23.1.3) still imports `VCRUNTIME140.dll` and the `api-ms-win-crt` DLLs

## Symptom

`package/music-player/desktop-app` builds the music player for Windows with
`rustflags = ["-C", "target-feature=+aes,+crt-static"]` for `x86_64-pc-windows-msvc` in its
`.cargo/config.toml`, which also pins `linker = "lld-link.exe"` for that target.
The intent of `+crt-static` is an executable that needs no Visual C++ Redistributable.

The release workflow (`.github/workflows/music-player-release.yml`) reads the executable's import tables with `pefile`
(an inline Python script over the import and the delay-import directory)
and refuses an executable that imports a Visual C++ runtime DLL.
On the `windows-2025` runner (image `windows-2025-vs2026` version `20260925.250.1`) the x64 executable failed
that check in run 38662748558:

```text
imports: advapi32.dll api-ms-win-core-synch-l1-2-0.dll api-ms-win-core-winrt-error-l1-1-0.dll api-ms-win-crt-heap-l1-1-0.dll api-ms-win-crt-locale-l1-1-0.dll api-ms-win-crt-math-l1-1-0.dll api-ms-win-crt-runtime-l1-1-0.dll api-ms-win-crt-stdio-l1-1-0.dll api-ms-win-crt-string-l1-1-0.dll bcrypt.dll bcryptprimitives.dll combase.dll comctl32.dll dwmapi.dll dwrite.dll gdi32.dll imm32.dll kernel32.dll mmdevapi.dll ntdll.dll ole32.dll oleaut32.dll opengl32.dll propsys.dll shell32.dll shlwapi.dll uiautomationcore.dll user32.dll uxtheme.dll vcruntime140.dll
##[error]The executable imports a Visual C++ runtime DLL, so it is not self-contained: vcruntime140.dll
```

An earlier check that searched the file for strings flagged `VCRUNTIME140.dll` too, and was replaced by the `pefile`
check because a string in the file is not an import.
The `windows-11-arm` job (arm64, default MSVC `link.exe`, same `+crt-static`) passed the same check.

## Root cause

**Not established.**
What the runs show is a correlation with the linker, and nothing more.

The probe run 38664446757 built only the x64 target and changed one thing: it exported
`CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=link.exe`, which overrides the `linker` key of the config file.
The `+crt-static` flag came from the same config file in both runs (the workflow sets no `RUSTFLAGS`).
The import list of that build:

```text
imports: advapi32.dll api-ms-win-core-synch-l1-2-0.dll api-ms-win-core-winrt-error-l1-1-0.dll bcrypt.dll bcryptprimitives.dll combase.dll comctl32.dll dwmapi.dll dwrite.dll gdi32.dll imm32.dll kernel32.dll mmdevapi.dll ntdll.dll ole32.dll oleaut32.dll opengl32.dll propsys.dll shell32.dll shlwapi.dll uiautomationcore.dll user32.dll uxtheme.dll
```

It holds no `vcruntime140.dll` and no `api-ms-win-crt-*` entry, and the job passed.
So with `+crt-static` fixed, switching `lld-link.exe` to `link.exe` removed the dynamic CRT imports on this runner.

Why is open.
No linker source, linker command line or object-file directive was inspected, so none of the following was tested:

- an object from a C or C++ dependency carries a `/DEFAULTLIB:` directive for the dynamic CRT that the two linkers
  resolve differently;
- the two linkers differ in how they pick the CRT import library when static and dynamic ones are both named;
- `+crt-static` is applied to rustc's own link arguments but not to something the build adds for a dependency.

A test that would separate them is to print the exact link line (`cargo rustc --release --target
x86_64-pc-windows-msvc -- --print link-args`, or the linker's `/VERBOSE` output) for both linkers and to list the
`/DEFAULTLIB:` directives of the dependency objects.

## Verification

- Failing: run 38662748558, job `build-desktop (windows, x64, x86_64-pc-windows-msvc, windows-2025)`, step
  `Package (Windows executable)`, output above.
- Working: probe run 38664446757, same job, one environment variable different.
- The script behind both: it parses the import and delay-import directories of the built `.exe` with `pefile` and
  prints the lower-cased DLL names.
- LLVM on the runner: `LLVM version: 23.1.3`, in `C:\Program Files\LLVM\bin`, which the job adds to `PATH`.
  The Rust toolchain was `nightly-2026-10-13`.
- Later full runs (38665954226 onward, with the override in the workflow) have the x64 job green with the check
  passing, and the artifact of run 38669432843 was parsed locally: console subsystem, system DLLs only.

Not tested: another LLVM version, another Rust version, a local Windows machine,
and whether the executable runs on a clean machine
(no Windows machine was available; the CI job only runs `--version`).

## Verified workarounds

The release workflow exports `CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=link.exe` for the x64 build.
Update 2026-10-15:
 that export is removed from the workflow,
 because `.cargo/config.toml` no longer sets a
`linker` key for the MSVC targets,
 so `link.exe` is the default (`doc/decision/music-player-windows-linkers.md`).
The paragraph below describes the state when the export existed.
The environment variable takes precedence over the `linker` key of the config file, so developers keep their
`lld-link.exe` pin.

Tradeoffs: the release binary is linked by a different linker than developers use,
so a defect visible only with one of them could differ between a developer build and a release;
`link.exe` comes from the Visual Studio installation of the runner image and is not pinned by this repository.
The import check stays in the job, so a future regression is caught.

## What does not work

- `+crt-static` alone with `lld-link.exe` (this document's symptom).
- A check that searches the executable's bytes for DLL names: it matches strings that are not imports.

## Upstream filing decision

`.out-of-scope/` was checked: no exemption covers the Rust linker or `lld`.
A search of `rust-lang/rust` issues for `crt-static lld-link vcruntime140`, and of all of GitHub for
`crt-static lld-link msvc vcruntime`, returned no matches.
That does not show that no report exists.

1.  Is it upstream's fault: not established; the cause is unknown.
2.  Can upstream fix it: unknown until the cause is known.
3.  Supported use case: `+crt-static` on MSVC is documented by Rust; use with `lld-link` was not looked up.
4.  Contribution welcome: not assessed.
5.  Likely to be fixed: not assessed.
6.  Prototype: none, because there is no root cause to patch.

No issue is drafted: a report would be a correlation without a cause.
Reopen when the link line has been captured for both linkers.

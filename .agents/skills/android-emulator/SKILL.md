---
name: android-emulator
description: >-
  Android emulator or AVD work. Use before launching, scripting or briefing an agent
  to use the Android emulator (device passes, screenshots, gfxinfo, adb on an emulator).
  Loading it is a required step before the first launch. Headless launches are banned.
---

# Android emulator

## Launch

EMW:
 Android emulators run headful on the host GPU as their own systemd user unit;
 never `-no-window`,
 `xvfb-run`,
 or a container (headless launches crashed and hung for hours).
Launch fails:
 stop and report the error.

EMB:
 Briefs for agents that use an emulator repeat the headful rule and name the launch form in
 `doc/troubleshooting/android-emulator-37-software-renderer-sigsegv.md`;
 a brief never permits `-no-window`.

The launch form is the section "Working agent launch: headful, host GPU, no container" of that document:
`systemd-run --user --unit=<name> --collect --property=MemoryMax=12G` with the display environment set,
`-gpu host`, a private reflink copy of the owned AVD, and its own adb port.
Read the section before writing the command;
it lists the `HOME`, `ANDROID_USER_HOME` and `ANDROID_AVD_HOME` the copy needs.

## Use and shutdown

EMA:
 Always allow USB debugging on an emulator:
 accept the authorization dialog with always allow from this computer ticked,
 and never ask the user to approve it.

EMS:
 Never run `wm density` or `wm size` on an emulator;
 change display size through Settings,
 record settings before the pass,
 and restore them after.

EMD:
 Shut an emulator down gracefully:
 `sync`,
 wait 8 seconds,
 `sync`,
 then `adb emu kill`;
 never kill the unit while the guest writes.

EMP:
 Drive a private copy of the owned AVD,
 never the owned AVD itself,
 with captures of invented libraries only (no private names or status bars).

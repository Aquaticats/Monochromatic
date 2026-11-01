# @monochromatic-dev/android-exempt-unused-cli

Interactive CLI to exempt (and un-exempt) third-party Android apps from
permission auto-revoke,
 over adb.

Android revokes permissions from apps you have not used for a while.
 This tool
sets each app's `AUTO_REVOKE_PERMISSIONS_IF_UNUSED` appops mode so chosen apps
keep their permissions.

## What it replaces

The one-shot bash original exempts every third-party app at once:

```bash
adb shell 'for p in $(pm list packages -3 | sed "s/package://"); do cmd appops set $p AUTO_REVOKE_PERMISSIONS_IF_UNUSED ignore; done'
```

This CLI turns that into a reviewable,
 two-way operation.

## How it works

The multiselect is a live mirror of device state,
 not a blank checklist:

- Every third-party app is listed.
- Apps already exempted are pre-checked and hinted `currently exempted`.
- You edit the checkboxes,
   then confirm.
- The tool diffs your selection against the device and applies both directions:
  newly checked apps are exempted (`ignore`);
   unchecked apps that were exempted
  are reverted to default (auto-revoke re-enabled).

There is no separate exempt-vs-revert mode:
 the checkbox state is the intent.

## Prerequisites

- `adb` on `PATH` (Android platform-tools).
- A device (or emulator) with USB debugging enabled and authorized;
   confirm with
  `adb devices`.

When more than one device is connected,
 the CLI prompts you to pick one.

## Usage

```bash
# from the repo (quoted: the bare task path's "-unused:run" tail otherwise
# matches the repo deny-list's curl basic-auth credential rule)
mise run '//package/android-exempt-unused/cli:run'

# or, once built and linked, via its bin
android-exempt-unused
```

## Notes

- Reading current state uses `cmd appops query-op` when available,
   falling back
  to per-app `cmd appops get` on older Android versions.
- Applying changes mutates device state.
   Exercise it against a throwaway
  emulator or test device before a primary phone.

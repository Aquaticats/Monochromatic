# Android raises a "not responding" dialog over a study under host load

## Symptom

On 2026-10-05 an Android 17 Pixel 9 Pro Fold emulator,
bounded to 6 GiB of memory and 2 CPUs,
ran on a host whose load average was between 50 and 100 from other work.
While a debug study activity was being captured,
the system put its own dialog in front of it:

```text
# adb shell dumpsys window displays
mCurrentFocus=Window{e7a56fd u0 Application Not Responding: com.google.android.apps.nexuslauncher}
```

The hierarchy then held only the dialog's texts,
`Pixel Launcher isn't responding`,
`Close app` and `Wait`,
so a capture that looked for the study's own text failed with a message
about the study,
not about the dialog.

## What was tried

- Tapping `Wait` and launching the study again,
  up to four times per state:
  the dialog was back each time,
  and the capture gave up.
- Waiting for the host:
  when the load average fell under 1 the same capture met no dialog.
  When the load rose again during that visit,
  to between 70 and 96,
  the captures that completed still met none.

So the dialog is not tied to load alone;
what else decides it was not established.

## What helps

- Check the focused window before trusting a hierarchy,
  and report a system dialog as what it is.
- Keep what was seen (hierarchy,
  screenshot,
  window dump),
  answer the dialog,
  and repeat the affected state within a bound.
- Keep each finished state as it completes,
  so a later attempt continues and does not start over.

## Related: the navigation bar is not reported right after boot

On the same day,
the first capture of a visit failed because `dumpsys window` listed a
`statusBars` inset source and no `navigationBars` source at all.
An earlier study's first capture of a visit had the source but drew no
gesture handle in it.
Both were the first view after boot and after the study's package was installed.
The capture now waits,
within a bound,
until the navigation source is reported,
waits five seconds more when it had to wait at all,
and then reads the hierarchy again.
The next visit's first view was captured with the handle drawn.
What delays the bar was not established.

## Related: a launch reported as `Status: timeout`

On 2026-10-06,
with the host's load average at about 60 to 70 from other work,
`am start -W` for the template editor host printed:

```text
Status: timeout
LaunchState: UNKNOWN (-1)
Activity: dev.monochromatic.musicplayer/.TemplateEditorActivity
WaitTime: 10572
Complete
```

The app's own log showed the activity created once with the requested scene,
so the launch had happened;
the activity manager stopped waiting for its first frame.
A capture that required `Status: ok` refused it,
and two visits in a row captured nothing new.
The capture now accepts `Status: timeout` and logs it,
because the creation count it reads next still requires exactly one creation
and every view is checked from its own hierarchy and frame.

## Not verified

A global setting named `hide_error_dialogs` is an idea from memory for
suppressing such dialogs.
It was not checked against Android's sources and not tried here.

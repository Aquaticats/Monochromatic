# Android Settings 17 shell font scale writes can exceed the user-facing preset range

## Symptom

A Pixel 9 Pro Fold Android 17 emulator accepts this command and reads the same value
back:

```text
adb -s emulator-5562 shell settings put system font_scale 0.5
adb -s emulator-5562 shell settings get system font_scale
0.5
```

A native app capture at that value renders text far smaller than ordinary Android UI.
The misleading conclusion is that 50% is a user-selectable Android text size.
 A 75%
value has the same problem.

The shell setting and the Settings UI answer different questions:

- The shell command proves the settings provider stores an arbitrary float.
- The Settings app exposes only values from its packaged preset resource.

## Root cause

The target emulator's installed package is
`/system_ext/priv-app/SettingsGoogle/SettingsGoogle.apk`,
 package
`com.android.settings`,
 version 17,
 version code 37.
 Its packaged
`array/entryvalues_font_size` resource contains:

```text
resource 0x7f0300a8 array/entryvalues_font_size
  () (array) size=7
    ["0.85", "1.0", "1.15", "1.30",
     "1.50", "1.80", "2.0"]
```

The Android Settings source loads that array as the complete list of available values.
Source:
 `aosp-mirror/platform_packages_apps_Settings`,
 commit
`7c598253ff60f06f8e6fe046f18fd88e9daa72d3`,
`src/com/android/settings/accessibility/FontSizeData.java:43-50`:

```java
final Resources resources = getContext().getResources();
final ContentResolver resolver = getContext().getContentResolver();
final List<String> strEntryValues =
        Arrays.asList(resources.getStringArray(R.array.entryvalues_font_size));
setDefaultValue(getFontScaleDefValue(resolver));
final float currentScale =
        Settings.System.getFloat(resolver, Settings.System.FONT_SCALE, getDefaultValue());
setInitialIndex(fontSizeValueToIndex(currentScale, strEntryValues.toArray(new String[0])));
setValues(strEntryValues.stream().map(Float::valueOf).collect(Collectors.toList()));
```

The Settings UI commits one value from that list rather than accepting free-form input.
Same source,
 `FontSizeData.java:54-63`:

```java
@Override
void commit(int currentProgress) {
    final ContentResolver resolver = getContext().getContentResolver();
    if (Settings.Secure.getInt(resolver,
            Settings.Secure.ACCESSIBILITY_FONT_SCALING_HAS_BEEN_CHANGED,
            /* def= */ OFF) != ON) {
        Settings.Secure.putInt(resolver,
                Settings.Secure.ACCESSIBILITY_FONT_SCALING_HAS_BEEN_CHANGED, ON);
    }
    Settings.System.putFloat(resolver, Settings.System.FONT_SCALE,
            getValues().get(currentProgress));
}
```

The earlier interpretation was wrong:
 a successful provider write does not add that
value to `entryvalues_font_size` and does not make it selectable in Settings.

## Verification

The target evidence came from the running Pixel 9 Pro Fold emulator,
 not a generic
Android assumption.

Identify and pull the installed package:

```text
adb -s emulator-5564 shell cmd package path com.android.settings
package:/system_ext/priv-app/SettingsGoogle/SettingsGoogle.apk

adb -s emulator-5564 pull \
  /system_ext/priv-app/SettingsGoogle/SettingsGoogle.apk \
  /home/user/temp/agent/target-android-settings.apk
```

Confirm package version:

```text
/home/user/Android/Sdk/build-tools/37.0.0/aapt2 dump badging \
  /home/user/temp/agent/target-android-settings.apk

package: name='com.android.settings' versionCode='37' versionName='17'
```

Read the target's preset resource:

```text
/home/user/Android/Sdk/build-tools/37.0.0/aapt2 dump resources \
  /home/user/temp/agent/target-android-settings.apk
```

### Values that represent the user-facing Settings control

- 0.85
- 1.0
- 1.15
- 1.30
- 1.50
- 1.80
- 2.0

### Values that the shell stores but this Settings UI does not expose

- 0.5
- 0.75

The accepted music-player screen was recaptured at every user-facing value.
 The mode
control uses one content-sized row at 85%,
 connected 2×2 from 100% through 130%,
then four connected rows from 150% through 200%.
 Every segment retains at least
12dp horizontal content padding.

## Verified workarounds

### Extract presets from the installed Settings package

Use `cmd package path`,
 `adb pull`,
 and `aapt2 dump resources` before defining a
device-specific text-scale matrix.

Tradeoff:
 the result describes that installed Settings build.
 Another device vendor or Android
release may package a different array and must be probed separately.

### Keep shell-only scales as explicit diagnostics

A shell-injected value can still test layout resilience outside the supported UI range.
Label it `developer-injected` and separate it from user-facing preset evidence.

Tradeoff:
 the diagnostic does not represent a state ordinary users can select through Settings.

## What does not work

### Treating `settings get` as a Settings UI capability probe

Reading back 0.5 confirms storage only.
 It does not enumerate the UI's allowed values.

### Inferring the target range from another Android release

Framework or AOSP defaults may resemble the installed Google Settings resource,
 but
they do not prove this emulator's package contents.
 Pulling the target APK removes that
assumption.

## Upstream filing decision

No `.out-of-scope/` entry covers Android Settings font scaling.
 Nothing should be filed
upstream:
 the observed behavior is not an Android defect.

1.  **Upstream fault:**
    no.
    The shell stored the requested float and the UI exposed its packaged preset list.
    The error was interpreting those as the same capability.
2.  **Upstream can fix it:**
    not applicable because no faulty behavior was found.
3.  **Supported use case:**
    Settings supports preset selection;
    arbitrary shell writes are a developer mechanism rather than a promise that every float appears in the UI.
4.  **Contribution welcome:**
    not evaluated because there is no defect or patch to contribute.
5.  **Likely upstream action:**
    not evaluated for the same reason.
6.  **Compatible minimal fix:**
    no upstream fix is appropriate.
    The consumer-side fix is to derive review scales from the installed resource.

No duplicate search or issue draft is needed because constraint one fails and there is
no upstream report to make.

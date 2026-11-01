# An Android RELRO collector rejected a valid dedicated LOAD layout

## Symptom

The research collector in native CI run `qvsjm5hmdp` rejected the packaged
`lib/arm64-v8a/libandroidx.graphics.path.so` after the Android build had succeeded.
Its assertion required every GNU RELRO region to end on a 16 KB boundary.
This was our collector's assertion,
not an Android linker diagnostic or an established AndroidX defect.

The first collector stopped after copying the debug APK.
Although the owning Gradle task had built both APKs successfully,
that artifact therefore lacked the AndroidTest APK.
The corrected run `1jxc80q7q7` retained both APKs before checking their contents.

## Root cause

The collector omitted the dedicated LOAD exception documented in Android's
[16 KB page-size guide][page-sizes].
The measured AndroidX layout has a writable LOAD starting at `0x5c40`,
with memory size `0x390`,
and a GNU RELRO region starting at the same address with size `0x3c0`.
RELRO covers that LOAD completely.
Its end is `0x6000`,
while the next writable LOAD starts at `0x9fd0`.
Rounding the protected end to `0x8000` does not touch that later writable region.

Current Bionic source corroborates why a modulus-only assertion is insufficient.
In the external `aosp-mirror/platform_bionic` checkout at
`731631f300090436d7f5df80d50b6275c8c60a93`,
`linker/linker_phdr.cpp:1294` finds the LOAD with the same starting virtual address:

```cpp
// External Bionic: linker/linker_phdr.cpp
if (phdr->p_type == PT_LOAD && phdr->p_vaddr == relro_phdr->p_vaddr) {
```

At `linker/linker_phdr.cpp:1326`,
the linker refuses to extend protection when RELRO does not cover the entire LOAD:

```cpp
// External Bionic: linker/linker_phdr.cpp
if (relro_phdr->p_memsz < phdr->p_memsz) {
  return;
}
```

The adjacent comment specifically warns that comparing page-rounded sizes instead
could incorrectly protect non-relocation writable data.
At `linker/linker_phdr.cpp:1376`,
the normal protection path rounds the touched pages before calling `mprotect`:

```cpp
// External Bionic: linker/linker_phdr.cpp
ElfW(Addr) seg_page_start = page_start(phdr->p_vaddr) + load_bias;
ElfW(Addr) seg_page_end = page_end(phdr->p_vaddr + phdr->p_memsz) + load_bias;
```

This source is explanatory evidence from a current public checkout.
It is not a claim that the inspected checkout exactly matches a particular device image.

## Verification

The frozen application source is `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
The corrected workflow is `8256d8bb9dd20b8c9b373c2c08b0a73560d93564`.
The [retained Android receipts][receipts] bind the source,
workflow,
actual APK bytes,
`readelf` output,
and native-member hashes.

The classifier was tested against:

- A boundary-aligned positive fixture.
- A dedicated LOAD positive fixture.
- The actual padded AndroidX layout.
- A negative fixture with ordinary writable data sharing the LOAD.
- A negative fixture whose rounded protection overlaps a later writable LOAD.
- A negative fixture without a matching LOAD.

All 10 packaged native libraries were inspected in the corrected run.
Every applicable check passed.
The parent also exercised the matching APK pair on an actual 16,384-byte-page,
API 37 device.
Its instrumentation assertions remain separate acceptance evidence:
14 of the frozen cohort's 16 cases passed and 2 failed.
Successful ELF classification does not erase those failures.

## Verified workaround

Keep the ordinary boundary-aligned rule,
then admit a dedicated LOAD only when the LOAD starts with RELRO,
RELRO covers its full memory extent,
and rounded protection does not overlap unrelated writable LOAD data.
Retain the inspected intervals and the reason for each accepted exception.
Copy all producer artifacts before running potentially failing assertions,
then record all member failures rather than abandoning the remaining inventory.

This is a static layout check.
The tradeoff is that it cannot prove successful library loading,
JNI behavior,
or playback on every supported device and ABI.
Those consumers need their own execution receipts.

## What does not work

- Requiring `(relroAddress + relroMemorySize) % 16384 === 0` unconditionally:
  the measured dedicated LOAD is a counterexample.
- Accepting every unaligned RELRO region:
  the shared-data and later-overlap negative controls reject that relaxation.
- Comparing only rounded LOAD and RELRO lengths:
  Bionic's source explains how this can protect unrelated writable data.
- Claiming the first artifact contained both APKs because Gradle built them:
  collection stopped before the second copy.
- Replacing actual APK and device consumers with a passing compiler exit code.

## Upstream filing decision

No Android or AndroidX defect was established,
so no upstream issue was filed.
The `.out-of-scope/` inventory and its contents were checked;
none exempts this collector investigation.
GitHub issue search for `repo:aosp-mirror/platform_bionic RELRO 16KB`
returned zero matches with `incomplete_results: false` on 2026-10-18.
That narrow mirror search is not a complete search of Android's issue tracker.
There is no additive upstream bug claim to send.

The required filing criteria were considered separately:

- Upstream fault:
  no.
  The observed failure belongs to our research collector.
- Upstream fixability:
  no upstream change is needed;
  current Bionic already implements the relevant distinction.
- Supported use case:
  yes,
  Android's public documentation explicitly covers 16 KB layouts and the exception.
- Contribution acceptance:
  not determined for an issue filing,
  because no upstream defect or requested contribution was established.
- Maintainer willingness:
  no rejection is inferred from the narrow empty mirror search;
  the requested behavior is already documented and implemented.
- Minimal fix prototype:
  an upstream patch is inapplicable.
  The actual local prototype corrected our classifier and passed the negative controls and rerun.

The retained filing artifact is explicitly unsuitable for submission:

~~~md
Title: Research collector omitted Android's dedicated LOAD RELRO exception
Labels: none

Do not file as-is: this is a local collector defect, not an upstream bug report.

The collector rejected an AndroidX LOAD at 0x5c40 with size 0x390 because the
covering RELRO at 0x5c40 with size 0x3c0 ends at 0x6000. The next writable LOAD
starts at 0x9fd0. Android's page-size documentation and Bionic's
linker/linker_phdr.cpp dedicated-LOAD check explain the accepted layout.

Reproduction: run the first source-bound Android collector, qvsjm5hmdp;
its retained receipt records the false assertion and exact ELF intervals.

Suggested fix: correct the disposable research classifier to admit the
dedicated-LOAD case while rejecting shared writable data and protection overlap.
Workflow 8256d8bb9d and run 1jxc80q7q7 implement and verify that local correction.
~~~

[page-sizes]: https://developer.android.com/guide/practices/page-sizes
[receipts]: ../planning/depot-complete-coverage-android-receipts.json.gz

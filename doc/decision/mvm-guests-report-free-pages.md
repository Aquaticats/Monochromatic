# mvm guests report their free pages to the host

Status: decided by the owner on 2026-11-25T18:58-05:00, answering "Should mvm turn on free page reporting for every guest it
creates?" with "Turn it on (Recommended)".

## What was decided

Every libvirt guest `mvm` creates carries a virtio memory balloon with free page reporting on:

```xml
<!-- package/mvm/cli: the device mvm's domain definition gains -->
<memballoon model="virtio" freePageReporting="on"/>
```

No round's test steps change.

## Why

`mvm` writes no `memballoon` element, so libvirt adds a plain virtio balloon and QEMU leaves its `free-page-reporting`
property at its default, off. A guest's freed memory then stays charged to the host. Measured once each on this host
(libvirt 12.9.0, QEMU 11.1.3) with an Ubuntu 24.04 guest after one build of the git wrapper: the default guest stayed charged
7.98 GiB at rest, the guest with reporting on 5.52 GiB, 2.46 GiB less
(`doc/planning/mvm-guest-memory-held-for-a-rounds-life.md`, "Free page reporting on, the same guest with one domain attribute").

## Limits, stated with the decision

- Each figure is one run, under different host pressure; the run-to-run spread is unmeasured.
- The page-fault cost when a guest reuses memory it returned is unmeasured.
- Reporting returns nothing while a build's pages are live, so a build's peak is unchanged.
- The proposal's other options (a smaller guest running binaries the host built, dropping the guest's cache after an arm,
  destroying a guest between arms, KSM) are not decided here; its gate review
  found the smaller-guest option unmeasured for the arms the repository runs.

## Options that were not chosen

- Measure first: a round would have measured the spread and the fault cost before any change. Not chosen: guests keep
  holding freed memory meanwhile.
- Leave it off: no change to `mvm`. Not chosen: it leaves the measured saving unused.

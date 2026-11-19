# A reboot-free Linux CPU temperature target

Status: **unapproved** research, recorded 2026-11-18T21:12:44-05:00.
This is a feasibility finding, not an implementation or a tested operating limit.

## Answer

A vendor-neutral Linux tool can **aim for** a CPU temperature by reading a CPU sensor and adjusting CPUFreq policy
maxima while Linux is running. This host exposes both interfaces. The mechanism does not require a reboot to change
or turn off the target. It does **not** establish that the CPU will never cross the selected temperature.

- `setMaxCpuTemp 80` would enable or change a live 80 °C target for a named sensor or aggregate. A controller would
  lower frequency limits as that reading approaches the target and relax them when there is headroom. It would report
  when it cannot control the reading, not silently treat the target as a ceiling.
- `setMaxCpuTemp 95` would still be an enabled target. AMD lists **95 °C as the Ryzen 9 7950X's maximum operating
  temperature (`Tjmax`)** in its [processor specifications][amd-7950x]. AMD also lists temperature as one of the
  inputs to [Precision Boost 2][amd-precision-boost], which raises frequencies when the workload calls for it and
  there is headroom. A 95 °C target may intervene less than an 80 °C target, but a controller that acts before its
  target can still intervene. AMD's specification is neither an `off` sentinel nor a guarantee for every Linux
  sensor reading, and it does not make 95 °C a portable default.
- `setMaxCpuTemp off` would stop the feedback loop and undo only its own frequency restrictions **without rebooting**.
  It would leave independent restrictions, the power profile, and firmware protections alone. Success would mean
  that the tool removed its restrictions, not simply that its process stopped or that it wrote the hardware maximum.

## What Linux exposes on this host

- **Temperature input.** The read-only probe found `k10temp` `Tctl`, `Tccd1`, and `Tccd2` at `temp1_input`,
  `temp3_input`, and `temp4_input`, respectively. The [hwmon ABI][hwmon-temps] defines `temp*_input` in millidegrees
  Celsius for temperature channels. The [k10temp documentation][k10temp] distinguishes `Tctl`, the cooling control
  value, from die temperature where the driver reports them separately, and describes `Tccd*` as chiplet readings.
  [The driver returns mode `0444`][k10temp-files] for the attributes it exposes. A tool can pick `Tctl` or explicitly
  target the hottest of the three readings, as the prior experiment did. Those are different policies; neither is
  established here as AMD's internal junction-limit signal. Sensor selection must be checked on each supported host,
  not inferred from an `hwmon` index or a fixed channel number.
- **Frequency control.** This Ryzen 9 7950X reports `amd-pstate-epp` in active mode and exposes 32 CPUFreq policies,
  each with a root-owned `0644` `scaling_max_freq`. [CPUFreq][cpufreq-limit] defines that value as the policy maximum
  in kHz. A policy may cover multiple CPUs, so host-wide control must cover all relevant `policy*` entries, not write
  only to CPU 0 or assume that one policy always means one core. The [CPUFreq write handler][cpufreq-write] updates
  the live frequency QoS request, and the [active AMD driver][amd-epp-update] passes performance limits to CPPC.
  [Active-mode documentation][amd-active] describes EPP as a preference and firmware as choosing the operating
  frequency. Neither a frequency maximum nor an EPP preference is a temperature setting. The portable part is the
  CPUFreq and hwmon interface; a supported machine still needs a suitable sensor and a driver exposing usable limits.
- **Kernel thermal zones and power caps.** The probe counted 33 cooling devices but no `thermal_zone*` entries.
  The [thermal sysfs ABI][kernel-thermal] exposes trip points on zones, separately from cooling devices. The presence
  of cooling devices does not provide a CPU temperature trip point to set here. The observed powercap zones exposed
  `energy_uj` counters but no `constraint_*_power_limit_uw` files. The [powercap ABI][powercap] defines these as
  separate monitoring and limit attributes. The counter does not provide a writable power limit, and a watts limit,
  even where available, is not a Celsius target. Neither finding rules out firmware thermal management.

## What the evidence does not guarantee

A control loop reacts to past samples, not to the next workload burst. The temperature can continue rising after a
lower frequency limit is requested, and the available range may not be sufficient to keep the selected reading at a
given target. Even the kernel's [power allocator documentation][thermal-overshoot] distinguishes a desired temperature
from conditions that cause overshoot. The live-control path is documented, but it does not establish a response time,
a safe sampling interval or margin, or a maximum excursion for this machine.

The [existing handover](../handover/claude-cpu-cap-and-thermal-watch.md), lines 1966 to 2064,
records a prior `cpu.max` experiment over selected user cgroups. Of 5,257 samples, four met or exceeded 80 °C,
all while the profile was `balanced`. The peak was 84.0 °C. After the switch to `power-saver`, the peak across
5,078 samples was 66.0 °C. The quota controller remained in use and the workload was not held constant.
This shows an overshoot and a post-switch association, not what a CPUFreq controller would do or what the profile
alone caused. The repository's
[Claude watcher](../../package/claude-cpu-cap/daemon/README.md) already samples temperatures but applies a quota to
the Claude tool cgroup, as the [runbook](../runbook/claude-cpu-cap.md#what-enforces-the-limits) describes.
[`cpu.max`][cpu-max] limits a group's CPU bandwidth. CPUFreq policy maxima instead affect workloads running on the
CPUs in those policies, whether or not they belong to that group.

A firmware setting that can only be changed in UEFI setup and applied by rebooting cannot satisfy either a live
target change or a live `off`. Whether a particular MSI setup offers such a setting is not needed for this finding.
No reboot-free firmware temperature setter was established here. The proposed Linux mechanism does not depend on one.

## Verification limits

The research agent inspected upstream Linux documentation and source, AMD's published specifications and boost
behavior, read-only host sysfs on `7.3.5-1-cachyos`, and the repository's prior records. The main agent corroborated
policy permissions, the thermal and powercap inventory, and the historical sample counts. No frequency, power,
firmware, or existing workload cgroup setting was changed. No synthetic load or temperature controller was run.
The upstream source links are for Linux `v7.3`, not an audit of the installed CachyOS patch set.

Live write acceptance, effective frequency response, overshoot under real workloads, sensor selection, and reliable
release of the limits remain unverified. Release needs particular care: [the CPUFreq read handler][cpufreq-read]
reports the policy maximum, while the write handler changes a request. A snapshot of an effective cap is not proof
that it is the original request. Any implementation would need to define ownership when other software manages
limits, and verify target changes and explicit `off` on a running machine before making operational claims.

[amd-7950x]: https://www.amd.com/en/products/processors/desktops/ryzen/7000-series/amd-ryzen-9-7950x.html
[amd-precision-boost]: https://www.amd.com/en/resources/support-articles/faqs/CPU-PB2.html
[hwmon-temps]: https://docs.kernel.org/hwmon/sysfs-interface.html#temperatures
[k10temp]: https://docs.kernel.org/hwmon/k10temp.html#:~:text=Tctl%20is%20the%20processor%20temperature%20control%20value
[k10temp-files]: https://github.com/torvalds/linux/blob/v7.3/drivers/hwmon/k10temp.c#L291-L337
[cpufreq-limit]: https://docs.kernel.org/admin-guide/pm/cpufreq.html#:~:text=Maximum%20frequency%20the%20CPUs%20belonging%20to%20this%20policy%20are%20allowed%20to%20be%20running%20at
[cpufreq-write]: https://github.com/torvalds/linux/blob/v7.3/drivers/cpufreq/cpufreq.c#L734-L750
[cpufreq-read]: https://github.com/torvalds/linux/blob/v7.3/drivers/cpufreq/cpufreq.c#L690-L701
[amd-epp-update]: https://github.com/torvalds/linux/blob/v7.3/drivers/cpufreq/amd-pstate.c#L1995-L2040
[amd-active]: https://docs.kernel.org/admin-guide/pm/amd-pstate.html#active-mode
[kernel-thermal]: https://docs.kernel.org/driver-api/thermal/sysfs-api.html#sysfs-attributes-structure
[powercap]: https://docs.kernel.org/power/powercap/powercap.html#power-zone-attributes
[thermal-overshoot]: https://docs.kernel.org/driver-api/thermal/power_allocator.html#pid-controller
[cpu-max]: https://docs.kernel.org/admin-guide/cgroup-v2.html#cpu-interface-files

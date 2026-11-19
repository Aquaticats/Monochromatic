# A reboot-free Linux CPU temperature target

Unapproved feasibility research recorded 2026-11-18T21:12:44-05:00.
The research and coordinating sessions checked upstream sources and read-only host state on `7.3.5-1-cachyos`.

## Command behavior

A privileged Linux service can read a CPU temperature through hwmon and update each CPUFreq policy's
`scaling_max_freq` while the machine is running. The CPUFreq write handler updates a live frequency QoS request.
Any host with a suitable sensor and a driver exposing CPUFreq limits can use these interfaces.
[Linux hwmon][hwmon-temps] · [Linux CPUFreq][cpufreq-limit] · [CPUFreq write handler][cpufreq-write]

- `setMaxCpuTemp 80` would set or change the target in the running service. The controller can start lowering the
  frequency ceiling before the reading reaches 80 °C and relax it gradually as the CPU cools.
- `setMaxCpuTemp 95` would request a 95 °C target. AMD specifies 95 °C as this 7950X's maximum operating temperature.
  The service could still reduce frequency as the reading approaches that target.
  [AMD Ryzen 9 7950X specifications][amd-7950x]
- `setMaxCpuTemp off` would release the service's restrictions live. The service needs clear ownership of the
  frequency settings it changes. The CPUFreq read handler reports the effective policy maximum and the write handler
  updates one request, so restoring a snapshot can inadvertently make another restriction persist.
  [CPUFreq read handler][cpufreq-read] · [CPUFreq write handler][cpufreq-write]

Sampling delay, thermal inertia and the available frequency range can lead to brief overshoot.
The controller should report when the CPU remains above target at its lowest allowed ceiling.
Workload tests would establish the observed overshoot, frequency response and release behavior on this machine.
[Linux thermal power allocator][thermal-overshoot]

## Workstation evidence

- The read-only probe of the 7950X found `k10temp` exposing `Tctl`, `Tccd1` and `Tccd2`.
  A portable implementation can select CPU sensor inputs by hardware and channel identity.
  The previous experiment targeted the highest value from the temperature control and chiplet inputs.
  [Linux k10temp][k10temp] · [Linux hwmon][hwmon-temps]
- All 32 CPUFreq policies expose a root-owned `0644` `scaling_max_freq` and currently report
  `scaling_min_freq` of `3012481` kHz. The `amd-pstate-epp` driver is in active mode and passes performance
  limits to the hardware through CPPC. Applying the limit across all policies covers the host.
  [Linux AMD P-State][amd-active] · [AMD driver source][amd-epp-update]
- The probe found 33 cooling devices and zero thermal zones. Powercap exposes energy counters and zero
  `constraint_*_power_limit_uw` files. The available Linux interfaces for the proposed loop are hwmon and CPUFreq.
  [Linux thermal sysfs][kernel-thermal] · [Linux powercap][powercap]
- An earlier experiment with a CPU quota peaked at 84.0 °C against an 80 °C goal. After switching to
  `power-saver` with the quota controller still running, the peak was 66.0 °C across 5,078 samples.
  The workload varied during the experiment. The existing Claude watcher uses `cpu.max` to cap one workload cgroup.
  [Experiment record][earlier-experiment] · [Claude watcher][claude-watcher] · [Linux cgroup CPU limits][cpu-max]

Live sysfs writes, sustained CPUFreq-based temperature regulation, and removal of the service's frequency limits
remain untested. The code references are from upstream Linux `v7.3`. Verification of the installed driver was read-only.

## Estimated Rust and Clap size

This estimate counts nonblank, hand-written lines including comments. It assumes one Rust binary,
a privileged systemd service, a configured hwmon sensor, and a root-owned target file that the CLI replaces
atomically. The service is the sole userspace writer of the CPUFreq ceiling it manages.

- 60 to 90 Rust lines for Clap, target parsing, and the live target file.
- 100 to 150 Rust lines to locate and read sensors and discover and update CPUFreq policies.
- 50 to 70 Rust lines for temperature feedback, ceilings, and hysteresis.
- 120 to 160 Rust lines for the daemon, release and restart handling, partial write failures, and diagnostics.

The estimate is 330 to 470 production Rust lines, 160 to 240 test lines, and 50 to 80 lines of packaging
and systemd configuration. That gives 540 to 790 owned lines before documentation.
The test budget covers threshold behavior, fixture sysfs reads and writes, release, and failed writes.

Nearby code provides scale. The file-enforcer argument excerpt at lines 43 to 104 has 58 nonblank lines,
the bash-limit Linux resource controls have 114, and the scope registry that updates live cgroup limits has 188.
[Clap arguments][clap-arguments] · [Linux resource controls][linux-limits] · [Live limit registry][limit-registry]

## Implemented controller

At `717c0cdae0` on 2026-11-18T21:51:57-05:00 the Rust and Clap package includes the live target command,
sensor selection, CPUFreq feedback, and an installable systemd unit. The implementation has 449 nonblank,
noncomment lines of production Rust and 931 with comments. The test source has 211 nonblank lines.

Clippy, the release build, and all five nextest cases passed. The staged command accepted `80`, `95`, and
`off`, preserved the request file on invalid input, and ran empty-journal cleanup. `systemd-analyze verify`
checked the staged unit. Workstation actuation and thermal tuning remain open.
[Package and installation instructions][implementation]

[amd-7950x]: https://www.amd.com/en/products/processors/desktops/ryzen/7000-series/amd-ryzen-9-7950x.html
[hwmon-temps]: https://docs.kernel.org/hwmon/sysfs-interface.html#temperatures
[k10temp]: https://docs.kernel.org/hwmon/k10temp.html#:~:text=Tctl%20is%20the%20processor%20temperature%20control%20value
[cpufreq-limit]: https://docs.kernel.org/admin-guide/pm/cpufreq.html#:~:text=Maximum%20frequency%20the%20CPUs
[cpufreq-write]: https://github.com/torvalds/linux/blob/v7.3/drivers/cpufreq/cpufreq.c#L734-L750
[cpufreq-read]: https://github.com/torvalds/linux/blob/v7.3/drivers/cpufreq/cpufreq.c#L690-L701
[amd-epp-update]: https://github.com/torvalds/linux/blob/v7.3/drivers/cpufreq/amd-pstate.c#L1995-L2040
[amd-active]: https://docs.kernel.org/admin-guide/pm/amd-pstate.html#active-mode
[kernel-thermal]: https://docs.kernel.org/driver-api/thermal/sysfs-api.html#sysfs-attributes-structure
[powercap]: https://docs.kernel.org/power/powercap/powercap.html#power-zone-attributes
[thermal-overshoot]: https://docs.kernel.org/driver-api/thermal/power_allocator.html#pid-controller
[earlier-experiment]: ../handover/claude-cpu-cap-and-thermal-watch.md
[claude-watcher]: ../../package/claude-cpu-cap/daemon/README.md
[cpu-max]: https://docs.kernel.org/admin-guide/cgroup-v2.html#cpu-interface-files
[clap-arguments]: ../../package/file-enforcer/cli/src/args.rs
[linux-limits]: ../../package/codex-plugin/bash-limit/src/limits.rs
[limit-registry]: ../../package/wg/cli-quicker-exempt/src/scope_registry.rs
[implementation]: ../../package/cpu-temperature/cli/README.md

# Set a live CPU temperature target

`setMaxCpuTemp` changes the target of a Linux system service that adjusts CPUFreq ceilings.

```bash
# Terminal on a host with the service installed and running.
sudo setMaxCpuTemp 80
sudo setMaxCpuTemp 95
sudo setMaxCpuTemp off
```

Temperatures are whole degrees Celsius. A numeric target enables feedback.
`off` releases the service's CPUFreq ceilings. The running service reads each change on its next pass.
Temperature can briefly exceed the target. At the available frequency floors, the service reports readings
that stay too hot in its systemd journal.

## Build and install

Build from the repository root.

```bash
# Monochromatic/
mise run //package/cpu-temperature/cli:build
```

Select CPU sensor inputs by hwmon driver name and temperature label or input filename.
Separate multiple selectors with commas. The service uses the highest selected reading.
For the Ryzen workstation, the configuration is:

```bash
# /etc/set-max-cpu-temp.conf
CPU_TEMPERATURE_SENSORS=k10temp:Tctl,k10temp:Tccd1,k10temp:Tccd2
```

The kernel must expose the selected sensors under `/sys/class/hwmon` and CPUFreq policies under
`/sys/devices/system/cpu/cpufreq`. Give the service sole ownership of userspace writes to the policies'
`scaling_max_freq`. Releasing a policy writes its hardware maximum so independent kernel limits can still apply.

Install the binary and the unit after creating the sensor configuration. The installed command name is
`setMaxCpuTemp` and the Cargo binary name is `set-max-cpu-temp`.

```bash
# Monochromatic/
sudo install --mode=755 package/cpu-temperature/cli/target/release/set-max-cpu-temp /usr/local/bin/setMaxCpuTemp
sudo install --mode=644 package/cpu-temperature/cli/systemd/set-max-cpu-temp.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now set-max-cpu-temp.service
```

## Sampling and release

The watcher begins reducing frequency three degrees before the target, reduces faster at the target,
and raises it gradually after cooling six degrees below the target. The starting sample interval is
500 ms. The earlier four-second quota experiment crossed its target. Margin and interval can be adjusted
in the service command with `--margin` and `--poll-ms`. Peak temperature and performance depend on the
host and workload. [Temperature experiment](../../../doc/research/linux-cpu-temperature-target.md)

The service requests the available frequency floors when a selected temperature input fails
and retries sensor discovery on later passes. Read and write diagnostics are available with
`journalctl --unit=set-max-cpu-temp.service`.

A root-only directory under `/run` holds the target and a release record for each affected policy.
Stopping the service runs cleanup using those records. A failed release leaves its record for retry.
Starting the service again uses the last target saved on that boot.

## Verification

The nextest suite exercises target changes, sensor loss and recovery, policy discovery, and release
against disposable sysfs fixtures. Workstation measurements and live kernel write verification are open.

```bash
# Monochromatic/
mise run //package/cpu-temperature/cli:test
```

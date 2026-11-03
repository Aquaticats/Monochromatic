# Kopia and pcloudcc persistent I/O weight

## Host configuration at 2026-11-03T02:21:04-05:00

The user requested that Kopia and pcloudcc permanently use I/O weight 10.
The host runs them through the user services `kopia-snapshot-all.service` and `pcloud.service`.
Applied `systemctl --user set-property` with `IOWeight=10` to each service,
without `--runtime`,
so systemd saved the setting for future boots and applied it to the running processes.

The persistent files are
`~/.config/systemd/user.control/kopia-snapshot-all.service.d/50-IOWeight.conf`
and `~/.config/systemd/user.control/pcloud.service.d/50-IOWeight.conf`.
Each contains `[Service]` with `IOWeight=10`.
Systemd generated these files;
use `systemctl --user set-property` for future changes.

Verification with `systemctl --user show` reported `IOWeight=10` for both services
and listed the persistent drop-ins.
Reading each service's cgroup files under `/sys/fs/cgroup` showed
that both `io.weight` and `io.bfq.weight` changed from `default 100` to `default 10`.
The Kopia PID remained 2998405 and the pcloudcc PID remained 5781;
neither service was restarted.
`StartupIOWeight` remains unset,
so the configured `IOWeight` also applies during system startup and shutdown,
as documented in the installed `systemd.resource-control(5)` manual.

The requested configuration is applied and persisted.
A reboot was not performed.

/**
 Init-system-specific cloud-init helpers for systemd and OpenRC.
 Generates runcmd and write_files blocks for enabling qemu-guest-agent
 and configuring serial console autologin on Linux guests.
 */

import type { InitSystem, } from './registry.ts';

//region Guest agent under a confining SELinux policy

/**
 Path of the systemd drop-in that starts the guest agent through `/usr/bin/env`.
 A distro that confines the agent with SELinux gets this file from cloud-init
 before the agent is enabled.
 */
export const GUEST_AGENT_DROP_IN_PATH =
  '/etc/systemd/system/qemu-guest-agent.service.d/mvm-unconfined.conf';

/**
 Write-files entry that starts `qemu-guest-agent` through `/usr/bin/env`.

 Fedora labels the agent binary `virt_qemu_ga_exec_t` and starts it from
 `/usr/lib/systemd/system/qemu-guest-agent.service`,
 whose unit file the policy turns into the `virt_qemu_ga_t` domain.
 That domain may not read the virtiofs share (`virtiofs_t`),
 write the guest's own files (`admin_home_t`),
 execute a file it wrote (`virt_qemu_ga_tmp_t`),
 or open a TCP connection,
 so every `mvm` route into the guest is refused.

 `/usr/bin/env` is labeled `bin_t`,
 which the policy lets the system manager turn into `unconfined_service_t`;
 `env` then executes the agent in that domain,
 with the same arguments the distro unit passes
 (`/usr/lib/systemd/system/qemu-guest-agent.service`).
 The two `EnvironmentFile` values stay unexpanded here on purpose:
 systemd expands them from `/etc/sysconfig/qemu-ga`,
 exactly as the distro unit does.

 @returns The `write_files` list entry, already indented for cloud-init

 @example
 ```ts
 guestAgentUnconfinedDropInEntry();
 // => '  - path: /etc/systemd/system/qemu-guest-agent.service.d/mvm-unconfined.conf\n    content: |\n      [Service]\n…'
 ```
 */
function guestAgentUnconfinedDropInEntry(): string {
  return `  - path: ${GUEST_AGENT_DROP_IN_PATH}
    content: |
      [Service]
      ExecStart=
      ExecStart=/usr/bin/env /usr/bin/qemu-ga --method=virtio-serial --path=/dev/virtio-ports/org.qemu.guest_agent.0 -F\${FSFREEZE_HOOK_PATHNAME} $QEMU_GA_ARGS`;
}

//endregion Guest agent under a confining SELinux policy

//region Systemd user-data helpers

/**
 Generates runcmd entries for enabling qemu-guest-agent on systemd distros.
 For a guest whose SELinux policy confines the agent,
 the same user-data first writes the drop-in that starts the agent through
 `/usr/bin/env`,
 so the first agent process already runs where it may read the virtiofs share
 and write and execute the guest's files.
 
 @param guestAgentIsSelinuxConfined - Whether this guest confines the agent

 @returns Cloud-init write_files and runcmd blocks for systemd
 
 @example
 ```ts
 templateRuncmdSystemd(false); // => 'runcmd:\n  - systemctl enable --now qemu-guest-agent\n'
 ```
 */
function templateRuncmdSystemd(guestAgentIsSelinuxConfined: boolean,): string {
  if (!guestAgentIsSelinuxConfined) {
    return `runcmd:
  - systemctl enable --now qemu-guest-agent
`;
  }
  return `write_files:
${guestAgentUnconfinedDropInEntry()}
runcmd:
  - systemctl daemon-reload
  - systemctl enable --now qemu-guest-agent
`;
}

/**
 Generates write_files and runcmd for serial autologin on systemd distros.
 Overrides the serial-getty service for ttyS0 with autologin for the given user.
 
 @param user - Login username for autologin
 
 @returns Cloud-init write_files and runcmd blocks
 
 @example
 ```ts
 vmAutologinSystemd('ubuntu');
 ```
 */
function vmAutologinSystemd(user: string,): string {
  return `write_files:
  - path: /etc/systemd/system/serial-getty@ttyS0.service.d/autologin.conf
    content: |
      [Service]
      ExecStart=
      ExecStart=-/sbin/agetty --autologin ${user} --noclear %I $TERM
      Restart=no
  - path: /home/${user}/.bash_logout
    owner: ${user}:${user}
    content: |
      echo ""
      echo "Session ended. Press Ctrl+] to disconnect."
runcmd:
  - systemctl daemon-reload
  - systemctl restart serial-getty@ttyS0.service
`;
}

//endregion Systemd user-data helpers

//region OpenRC user-data helpers

/**
 Generates runcmd entries for enabling qemu-guest-agent on OpenRC distros (Alpine).
 
 @returns Cloud-init runcmd block for OpenRC
 
 @example
 ```ts
 templateRuncmdOpenrc(); // => 'runcmd:\n  - rc-update add qemu-guest-agent\n  ...'
 ```
 */
function templateRuncmdOpenrc(): string {
  return `runcmd:
  - rc-update add qemu-guest-agent
  - service qemu-guest-agent start
`;
}

/**
 Generates write_files and runcmd for serial autologin on OpenRC distros (Alpine).
 Appends an agetty entry to `/etc/inittab` and sends SIGHUP to init.
 
 @param user - Login username for autologin
 
 @returns Cloud-init write_files and runcmd blocks
 
 @example
 ```ts
 vmAutologinOpenrc('alpine');
 ```
 */
function vmAutologinOpenrc(user: string,): string {
  return `write_files:
  - path: /etc/inittab
    append: true
    content: |
      ttyS0::respawn:/sbin/agetty --autologin ${user} 115200 ttyS0 vt100
  - path: /home/${user}/.ash_logout
    owner: ${user}:${user}
    content: |
      echo ""
      echo "Session ended. Press Ctrl+] to disconnect."
runcmd:
  - kill -HUP 1
`;
}

//endregion OpenRC user-data helpers

//region Init system dispatch

/**
 Dispatches to the correct template runcmd generator, {@link templateRuncmdOpenrc}
 or {@link templateRuncmdSystemd}, for the given init system.
 
 @param guestAgentIsSelinuxConfined - Whether the guest confines the agent with SELinux

 @param initSystem - Target init system

 @returns Cloud-init write_files and runcmd block
 
 @example
 ```ts
 templateRuncmd({ guestAgentIsSelinuxConfined: true, initSystem: 'systemd' });
 ```
 */
export function templateRuncmd({
  guestAgentIsSelinuxConfined,
  initSystem,
}: {
  readonly guestAgentIsSelinuxConfined: boolean;
  readonly initSystem: InitSystem;
},): string {
  if (initSystem === 'openrc')
    return templateRuncmdOpenrc();
  return templateRuncmdSystemd(guestAgentIsSelinuxConfined,);
}

/**
 Dispatches to the correct autologin generator, {@link vmAutologinOpenrc} or
 {@link vmAutologinSystemd}, for the given init system.
 
 @param initSystem - Target init system
 
 @param user - Login username
 
 @returns Cloud-init write_files and runcmd blocks
 
 @example
 ```ts
 vmAutologin({ initSystem: 'systemd', user: 'ubuntu' });
 ```
 */
export function vmAutologin({
  initSystem,
  user,
}: {
  readonly initSystem: InitSystem;
  readonly user: string;
},): string {
  if (initSystem === 'openrc')
    return vmAutologinOpenrc(user,);
  return vmAutologinSystemd(user,);
}

//endregion Init system dispatch

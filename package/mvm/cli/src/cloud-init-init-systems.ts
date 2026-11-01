/**
 Init-system-specific cloud-init helpers for systemd and OpenRC.
 Generates runcmd and write_files blocks for enabling qemu-guest-agent
 and configuring serial console autologin on Linux guests.
 */

import type { InitSystem, } from './registry.ts';

//region Systemd user-data helpers

/**
 Generates runcmd entries for enabling qemu-guest-agent on systemd distros.
 
 @returns Cloud-init runcmd block for systemd
 
 @example
 ```ts
 templateRuncmdSystemd(); // => 'runcmd:\n  - systemctl enable --now qemu-guest-agent\n'
 ```
 */
function templateRuncmdSystemd(): string {
  return `runcmd:
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
 
 @param initSystem - Target init system
 
 @returns Cloud-init runcmd block
 
 @example
 ```ts
 templateRuncmd('systemd');
 ```
 */
export function templateRuncmd(initSystem: InitSystem,): string {
  if (initSystem === 'openrc')
    return templateRuncmdOpenrc();
  return templateRuncmdSystemd();
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

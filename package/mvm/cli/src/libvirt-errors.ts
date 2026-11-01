/**
 Errors raised when libvirt itself cannot be reached through `virsh`,
 as opposed to a libvirt operation failing.

 @module
 */

import {
  type ToolCommand,
  VIRT_MANAGER_FLATPAK,
} from './libvirt-tools.ts';

//region Advice

/**
 Explains how to start libvirt's session daemon by hand for the way `virsh` runs on this host.

 mvm does not start the daemon: `virsh` starts it on demand, and it keeps
 running while a domain runs. Starting it by hand is for finding out why it
 does not come up.

 @param virsh - Command mvm runs `virsh` with

 @returns Advice text of several lines

 @example
 ```ts
 sessionDaemonAdvice({ argv: ['virsh'], origin: 'path' });
 ```
 */
export function sessionDaemonAdvice(virsh: ToolCommand,): string {
  /**
   Start command inside the virt-manager Flatpak.
   */
  const flatpakStart = `flatpak run --command=virtqemud ${VIRT_MANAGER_FLATPAK} --verbose`;
  /**
   Start command on a host with libvirt installed.
   */
  const hostStart = 'virtqemud --verbose';
  return [
    'virsh starts libvirt\'s session daemon on demand, so mvm does not start it.',
    'To see why it does not come up, start it by hand in the background, with its output sent to a file rather than a pipe:',
    ...(virsh.origin === 'flatpak' ? [`  ${flatpakStart}`,] : []),
    ...(virsh.origin === 'path' ? [`  ${hostStart}`,] : []),
    ...(virsh.origin === 'environment'
      ? [
        `  ${hostStart}`,
        `  or, for the virt-manager Flatpak: ${flatpakStart}`,
      ]
      : []),
    `mvm runs virsh as: ${JSON.stringify(virsh.argv,)}`,
  ].join('\n',);
}

//endregion Advice

//region Errors

/**
 `virsh` ran but could not connect to libvirt's session daemon.

 @example
 ```ts
 try {
   await virsh({ args: ['list', '--all'] });
 }
 catch (error) {
   if (error instanceof LibvirtSessionUnavailableError) console.error(error.message);
 }
 ```
 */
export class LibvirtSessionUnavailableError extends Error {
  /**
   @param cause - Failed virsh call, kept for its output

   @param stderr - Standard-error text virsh printed, quoted in the message

   @param virsh - Command mvm runs `virsh` with, used for the start advice
   */
  constructor({
    cause,
    stderr,
    virsh,
  }: {
    readonly cause: unknown;
    readonly stderr: string;
    readonly virsh: ToolCommand;
  },) {
    super(
      [
        'virsh could not connect to libvirt\'s session daemon:',
        stderr.trim(),
        sessionDaemonAdvice(virsh,),
      ].join('\n',),
      { cause, },
    );
    this.name = 'LibvirtSessionUnavailableError';
  }
}

/**
 `virsh` was still running at its deadline, which a healthy session daemon does not cause.

 @example
 ```ts
 try {
   await virsh({ args: ['list', '--all'] });
 }
 catch (error) {
   if (error instanceof LibvirtUnresponsiveError) console.error(error.message);
 }
 ```
 */
export class LibvirtUnresponsiveError extends Error {
  /**
   @param cause - Timed-out virsh call, kept for its command line

   @param seconds - Seconds virsh was given, named in the message

   @param virsh - Command mvm runs `virsh` with, used for the start advice
   */
  constructor({
    cause,
    seconds,
    virsh,
  }: {
    readonly cause: Error;
    readonly seconds: number;
    readonly virsh: ToolCommand;
  },) {
    super(
      [
        `virsh did not finish within ${String(seconds,)} seconds and was stopped; libvirt's session daemon is missing or not answering.`,
        cause.message,
        sessionDaemonAdvice(virsh,),
      ].join('\n',),
      { cause, },
    );
    this.name = 'LibvirtUnresponsiveError';
  }
}

//endregion Errors

/**
 Errors raised when libvirt itself cannot be reached through `virsh`,
 as opposed to a libvirt operation failing.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import { SESSION_DAEMON_UNIT, } from './config.ts';
import {
  type ToolCommand,
  VIRT_MANAGER_FLATPAK,
} from './libvirt-tools.ts';

//region Advice

/**
 Text `virsh` prints first when it cannot reach libvirt at all.
 */
export const CANNOT_CONNECT_TEXT = 'failed to connect to the hypervisor';

/**
 Explains how to start libvirt's session daemon by hand for the way `virsh` runs
 on this host.

 mvm starts the daemon itself, in a user unit of its own; starting it by hand
 is for finding out why it does not come up.

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
    'To see why libvirt\'s session daemon does not come up, start it by hand in the background, with its output sent to a file rather than a pipe:',
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

//region Session daemon errors

/**
 mvm could not get libvirt's session daemon answering in its own user unit.

 @example
 ```ts
 try {
   await ensureSessionDaemon();
 }
 catch (error) {
   if (error instanceof SessionDaemonStartError) console.error(error.unit);
 }
 ```
 */
export class SessionDaemonStartError extends Error {
  /**
   Unit name without `.service`.
   */
  readonly unit: string;

  /**
   Whether the unit name was still loaded at the last attempt, which means a unit
   of that name was running or shutting down and mvm retried the start until its
   deadline.
   */
  readonly unitStillLoaded: boolean;

  /**
   @param cause - Failed start or last failed check, kept for its text

   @param startCommandLine - Rendered start command, named in the message

   @param unitStillLoaded - Whether the last start was refused because the unit
   name is loaded

   @param virsh - Command mvm runs `virsh` with, used for the start advice
   */
  constructor({
    cause,
    startCommandLine,
    unitStillLoaded = false,
    virsh,
  }: {
    readonly cause?: unknown;
    readonly startCommandLine: string;
    readonly unitStillLoaded?: boolean;
    readonly virsh: ToolCommand;
  },) {
    super(
      [
        `mvm starts libvirt's session daemon in the user unit ${SESSION_DAEMON_UNIT}.service, and it is not answering:`,
        `  ${startCommandLine}`,
        ...(cause === undefined ? [] : [caughtValueText(cause,),]),
        ...(unitStillLoaded
          ? [
            'That unit name was still loaded every time mvm tried to start it, up to its deadline, which systemd reports for a unit that is running, shutting down, failed and not yet unloaded, or holds a fragment file.',
            'If it is active after its daemon exited, it holds processes (guests the daemon no longer manages), and stopping it kills them.',
          ]
          : []),
        'Read what the unit reported:',
        `  journalctl --user-unit=${SESSION_DAEMON_UNIT}.service`,
        `  systemctl --user status ${SESSION_DAEMON_UNIT}.service`,
        'If a unit of that name failed earlier and still holds the name, unload it:',
        `  systemctl --user reset-failed ${SESSION_DAEMON_UNIT}.service`,
        sessionDaemonAdvice(virsh,),
      ].join('\n',),
      { cause, },
    );
    this.name = 'SessionDaemonStartError';
    this.unit = SESSION_DAEMON_UNIT;
    this.unitStillLoaded = unitStillLoaded;
  }
}

/**
 The command that runs `virsh` says nothing about which command runs `virtqemud`.

 @example
 ```ts
 try {
   sessionDaemonCommand({ override: '', virsh: { argv: ['virt-wrapper'], origin: 'environment' } });
 }
 catch (error) {
   if (error instanceof SessionDaemonCommandError) console.error(error.message);
 }
 ```
 */
export class SessionDaemonCommandError extends Error {
  /**
   @param variable - Environment variable that names the daemon command

   @param virsh - Command mvm runs `virsh` with, quoted in the message
   */
  constructor({
    variable,
    virsh,
  }: {
    readonly variable: string;
    readonly virsh: ToolCommand;
  },) {
    super(
      [
        `mvm cannot tell which command runs libvirt's session daemon, because nothing in the command it runs virsh with (${JSON.stringify(virsh.argv,)}) says \`virsh\`.`,
        `Set ${variable} to a JSON array holding the executable that runs \`virtqemud\` and its leading arguments, for example ${variable}='["virtqemud"]'.`,
      ].join('\n',),
    );
    this.name = 'SessionDaemonCommandError';
  }
}

//endregion Session daemon errors

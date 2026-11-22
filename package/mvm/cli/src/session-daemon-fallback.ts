/**
 What mvm does when it has no systemd user manager to start libvirt's session
 daemon in, and the environment its `virsh` calls take from that decision.

 mvm starts the daemon in a unit of its own whenever it can. Two cases leave it
 no unit: no `systemd-run` on `PATH`, and `systemd-run --user` reporting that it
 cannot reach the user manager, which a caller whose `XDG_RUNTIME_DIR` is not the
 user manager's gets. There mvm falls back to letting `virsh` start the daemon the
 way it did before the unit existed, and warns about the cgroup defect that
 leaves, naming `doc/troubleshooting/libvirt-session-daemon-runs-in-the-capped-scope-of-whichever-command-starts-it.md`.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Names

/**
 Path of the document that records the cgroup defect a fallback leaves.

 @internal
 */
const TROUBLESHOOTING_DOC =
  'doc/troubleshooting/libvirt-session-daemon-runs-in-the-capped-scope-of-whichever-command-starts-it.md';

//endregion Names

//region The fallback

/**
 Why mvm could not put libvirt's session daemon in a unit of its own, and so let
 `virsh` start it the way it did before.

 - `no-systemd-run`: `systemd-run` is not installed, so there is no systemd user
   manager to start a unit in.
 - `user-manager-unreachable`: `systemd-run --user` answered that it cannot
   connect to the user bus, as it does when `XDG_RUNTIME_DIR` is not the one the
   user manager uses.
 */
export type SessionDaemonFallbackReason = 'no-systemd-run' | 'user-manager-unreachable';

/**
 The fallback this process took, when it took one; empty until then.
 */
const fallback: { reason?: SessionDaemonFallbackReason; } = {};

/**
 Text mvm warns with when it cannot use a unit of its own: why, what happens
 instead, and where the cgroup defect it leaves is recorded.

 @param reason - Why the unit could not be used

 @returns Warning text of several lines

 @example
 ```ts
 fallbackWarning('no-systemd-run');
 ```
 */
function fallbackWarning(reason: SessionDaemonFallbackReason,): string {
  /**
   Why the unit could not be started, in the reader's terms.
   */
  const cause = {
    'no-systemd-run': 'mvm found no systemd-run on PATH, so it cannot start libvirt\'s session daemon in a unit of its own here.',
    'user-manager-unreachable': 'mvm could not reach systemd\'s user manager, which systemd-run --user needs, so it could not start libvirt\'s session daemon in a unit of its own; a caller whose XDG_RUNTIME_DIR is not the user manager\'s gets this.',
  }[reason];
  return [
    cause,
    'For the rest of this process, libvirt\'s session daemon starts on demand inside the cgroup of the mvm command that asks for it, where that command\'s memory cap, time limit or cleanup kills the daemon and every guest of every session when that command ends.',
    'The unit\'s half-of-memory cap and its separation from that cgroup are not in place for those calls.',
    `Why that cgroup matters: ${TROUBLESHOOTING_DOC}`,
  ].join('\n',);
}

/**
 Records that this process let `virsh` start the session daemon and warns about
 what that costs.

 @param reason - Why no unit of mvm's own could be used

 @example
 ```ts
 takeFallback('no-systemd-run');
 ```
 */
export function takeFallback(reason: SessionDaemonFallbackReason,): void {
  /**
   Logger scoped to the fallback so the warning names where it happened.
   */
  const rl = tagged({
    tag: takeFallback.name,
    l,
  },);
  fallback.reason = reason;
  rl.warn(fallbackWarning(reason,),);
}

/**
 Environment for a `virsh` call that must not let libvirt start a session daemon
 on demand.

 `LIBVIRT_AUTOSTART=0` is the documented setting for asking without starting
 (libvirt 12.9, `src/libvirt.c`). After a fallback mvm asks for the opposite: the
 daemon does have to start on demand, and mvm states so rather than inherit a
 caller's `LIBVIRT_AUTOSTART=0`, because the warning it logged promises that
 start.

 @returns Entries to merge into the child's environment

 @example
 ```ts
 libvirtAutostartEnv(); // => { LIBVIRT_AUTOSTART: '0' }
 ```
 */
export function libvirtAutostartEnv(): Readonly<Record<string, string>> {
  return {
    LIBVIRT_AUTOSTART: fallback.reason === undefined ? '0' : '1',
  };
}

/**
 Environment every `virsh` call and the daemon question get: one language for
 libvirt's texts, and the on-demand decision from
 {@link libvirtAutostartEnv}.

 @returns Entries to merge into the child's environment

 @example
 ```ts
 libvirtEnv(); // => { LC_ALL: 'C', LIBVIRT_AUTOSTART: '0' }
 ```
 */
export function libvirtEnv(): Readonly<Record<string, string>> {
  return {
    // One language for libvirt's state names and error texts, which mvm reads.
    LC_ALL: 'C',
    ...libvirtAutostartEnv(),
  };
}

//endregion The fallback

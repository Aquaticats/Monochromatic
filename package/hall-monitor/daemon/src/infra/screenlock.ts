/**
 Screen-lock probe over the freedesktop ScreenSaver D-Bus interface.

 Skipping a cycle on a locked session is an optimisation, not a requirement, so
 this module owns the probe's failure rather than handing it to the daemon loop:
 a desktop without the interface, a host without `gdbus`, or a session bus
 restarting all mean "cannot tell", which costs one cycle instead of the
 process.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import spawn from 'nano-spawn';

import { log, } from './syslog.ts';

/**
 Module logger: the journal identity this package files every record under, then
 this module's subsystem tag.
 */
const l = tagged({
  l: log,
  tag: 'screenlock',
},);

/**
 Standard output of one session-bus call.
 */
type ScreenLockProbeResult = {
  /**
   Text `gdbus` printed for the call.
   */
  readonly stdout: string;
};

/**
 Asks the session bus whether the screen saver reports the session locked.
 */
type ScreenLockProbe = () => Promise<ScreenLockProbeResult>;

/**
 Asks the freedesktop ScreenSaver interface for its active state.

 @returns the call's standard output

 @throws when `gdbus` is missing, the session bus is unreachable, or the desktop
   does not implement the interface

 @example
 ```ts
 await probeScreenSaver(); // { stdout: '(false,)' }
 ```
 */
async function probeScreenSaver(): Promise<ScreenLockProbeResult> {
  /**
   Standard output of the session-bus call; the active state is read from it.
   */
  const { stdout, } = await spawn(
    'gdbus',
    [
      'call',
      '--session',
      '--dest',
      'org.freedesktop.ScreenSaver',
      '--object-path',
      '/org/freedesktop/ScreenSaver',
      '--method',
      'org.freedesktop.ScreenSaver.GetActive',
    ],
  );
  return { stdout, };
}

/**
 Checks whether the desktop session is locked via the freedesktop ScreenSaver
 D-Bus interface, which works on KDE, GNOME, and other compliant desktops.

 A probe that cannot answer is reported and read as an unlocked session, so a
 desktop without the interface or a session bus that is restarting costs one
 cycle rather than the daemon.

 @param probe - session-bus call; `gdbus` by default, and another one only so a
   test drives this module without a session bus

 @returns `true` when the session is locked, and `false` when it is unlocked or
   cannot be asked

 @example
 ```ts
 if (await isScreenLocked()) {
   log.debug('Screen locked, skipping cycle');
 }
 ```
 */
export async function isScreenLocked({
  probe = probeScreenSaver,
}: Readonly<{ probe?: ScreenLockProbe; }> = {},): Promise<boolean> {
  try {
    /**
     Session-bus answer; only the locked case skips a cycle.
     */
    const { stdout, } = await probe();
    // gdbus returns "(true,)\n" or "(false,)\n"
    return stdout.includes('true',);
  }
  catch (error: unknown) {
    // `warn`, not `reportExpectedFailure`: that helper rethrows a value which is not an Error, and a
    // probe failure must never end the daemon.
    l.warn(`Screen lock probe failed, reading the session as unlocked: ${caughtValueText(error,)}`,);
    return false;
  }
}

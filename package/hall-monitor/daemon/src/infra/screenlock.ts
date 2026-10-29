import spawn from 'nano-spawn';

/**
 Checks whether the desktop session is locked via the freedesktop
 ScreenSaver D-Bus interface (works on KDE, GNOME, and other compliant DEs).
 
 @returns `true` when the session is locked
 
 @example
 ```ts
 if (await isScreenLocked()) {
   log.debug("Screen locked, skipping cycle");
 }
 ```
 */
export async function isScreenLocked(): Promise<boolean> {
  /**
   Standard output from `gdbus`; parsed below for the boolean returned by the GetActive method.
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
  // gdbus returns "(true,)\n" or "(false,)\n"
  return stdout.includes('true',);
}

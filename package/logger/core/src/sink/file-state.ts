/**
 Resolve persistent per-user log locations without probing the filesystem at import. @module
 */
import { homedir, } from 'node:os';
import {
  posix,
  win32,
} from 'node:path';

//region State paths follow the host platform rather than the caller's cwd.
/**
 Resolve persistent logs using XDG state on Unix and conventional platform log directories elsewhere.

 @param platform - runtime platform, injectable for layout verification
 
 @param home - user-owned home, resolved only when fallback is attempted
 
 @param environment - runtime path overrides, isolated by consumers in disposable homes
 
 @returns Absolute directory for fallback JSONL files.

 @example
 ```ts
 const directory = userLogDirectory();
 ```
 */
export function userLogDirectory({
  platform = process.platform,
  home = homedir(),
  environment = process.env,
}: {
  readonly platform?: NodeJS.Platform;
  readonly home?: string;
  readonly environment?: Readonly<NodeJS.ProcessEnv>;
} = {},): string {
  if (platform === 'darwin')
    return posix.join(
      home,
      'Library',
      'Logs',
      'monochromatic',
    );
  if (platform === 'win32') {
    /**
     Runtime path override is accepted only when absolute.
     */
    const configured = environment.LOCALAPPDATA;
    /**
     Windows local application-data root, independent of cwd.
     */
    const local = (configured !== undefined) && win32.isAbsolute(configured,)
      ? configured
      : win32.join(
        home,
        'AppData',
        'Local',
      );
    return win32.join(
      local,
      'monochromatic',
      'Log',
    );
  }
  /**
   Runtime path override is accepted only when absolute.
   */
  const configured = environment.XDG_STATE_HOME;
  /**
   Persistent state root follows the XDG default for missing or invalid overrides.
   */
  const state = (configured !== undefined) && posix.isAbsolute(configured,)
    ? configured
    : posix.join(
      home,
      '.local',
      'state',
    );
  return posix.join(
    state,
    'monochromatic',
    'log',
  );
}
//endregion

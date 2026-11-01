/**
 Finding the nested compositor that mise has installed, without naming its version.
 
 The root `mise.toml` asks for the latest `cargo:monochromatic-nested-wayland-session`, so the runner uses whichever
 version mise has active and checks its capability instead (see `compositor-version.ts`). `mise where` prints the
 install directory of the active version. The runner starts the compositor from that absolute path: the mise shim on
 `PATH` needs the user's trusted config, which the systemd unit that hosts the compositor does not have.
 
 @example
 ```ts
  
 const path = await resolveCompositorPath({ cwd: process.cwd(), override: undefined, },);
 ```
 
 @packageDocumentation
 */

import {
  join,
  isAbsolute,
} from 'node:path';

import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import {
  type CommandResult,
  runCommand,
} from './command.ts';
import { expect, } from './errors.ts';

/**
 The mise tool name of the compositor, as in the root `mise.toml`.
 */
export const COMPOSITOR_TOOL = 'cargo:monochromatic-nested-wayland-session';

/**
 File name of the compositor executable inside the `bin` directory of its install.
 */
export const COMPOSITOR_EXECUTABLE = 'monochromatic-nested-wayland-session';

/**
 Turns the result of `mise where` into the compositor executable path.
 
 @param result - how `mise where cargo:monochromatic-nested-wayland-session` ended
 
 @returns the absolute path of the executable in the install directory
 
 @throws CheckFailureError when mise exited with an error, printed nothing, or printed something that is not an
 absolute path
 
 @example
 ```ts
 compositorPathFromMiseResult({ status: 0, stdout: '/m/installs/c/0.4.0', stderr: '', },); // '/m/installs/c/0.4.0/bin/...'
 ```
 */
export function compositorPathFromMiseResult(result: CommandResult,): string {
  /**
   The install directory mise printed.
   */
  const installDirectory = result.stdout
    .trim();
  expect({
    condition: result.status === 0,
    message: `"mise where ${COMPOSITOR_TOOL}" failed with status ${result.status} ("${result.stderr
      .trim()}"); `
      + `install the compositor with "mise install ${COMPOSITOR_TOOL}", or set E2E_COMPOSITOR to its executable`,
  },);
  expect({
    condition: isAbsolute(installDirectory,) && (!installDirectory.includes('\n',)),
    message: `"mise where ${COMPOSITOR_TOOL}" printed "${installDirectory}", not one absolute directory; `
      + 'set E2E_COMPOSITOR to the compositor executable',
  },);
  return join(
    installDirectory,
    'bin',
    COMPOSITOR_EXECUTABLE,
  );
}

/**
 Asks mise where the active compositor install is.
 
 @param cwd - directory to ask from, because the active version depends on the `mise.toml` files above it
 
 @returns the absolute path of the compositor executable
 
 @throws CheckFailureError when mise cannot say, see {@link compositorPathFromMiseResult}
 
 @example
 ```ts
 const path = await installedCompositorPath({ cwd: '/work/app', },);
 ```
 */
export async function installedCompositorPath({ cwd, }: { readonly cwd: string; },): Promise<string> {
  /**
   Tagged logger naming the lookup.
   */
  const rl = tagged({
    tag: installedCompositorPath.name,
    l: logger,
  },);

  /**
   How `mise where` ended.
   */
  const result = await runCommand({
    command: 'mise',
    args: [
      'where',
      COMPOSITOR_TOOL,
    ],
    cwd,
  },);
  rl.debug(`mise where answered status ${result.status} with "${result.stdout
    .trim()}"`,);
  return compositorPathFromMiseResult(result,);
}

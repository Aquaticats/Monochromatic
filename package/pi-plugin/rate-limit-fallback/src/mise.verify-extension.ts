/**
 Run the real Pi SDK without inherited credentials, user config, or project extensions.
 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import { mkdtempDisposable, } from 'node:fs/promises';
import { join, } from 'node:path';
import { text, } from 'node:stream/consumers';
import { fileURLToPath, } from 'node:url';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

/**
 Temporary HOME and working directory for a credential-free Pi process.
 */
await using home = await mkdtempDisposable(join(
  scratchRoot(),
  'pi-rate-limit-fallback-host-',
),);

/**
 Verifier child with only the disposable home and the parent's Node policy.
 */
const child = spawn(
  process.execPath,
  [fileURLToPath(new URL(
    'verify-pi-session.ts',
    import.meta.url,
  ),),],
  {
    cwd: home.path,
    env: {
      PATH: process.env
        .PATH,
      NODE_OPTIONS: process.env
        .NODE_OPTIONS,
      HOME: home.path,
      XDG_CONFIG_HOME: home.path,
      XDG_CACHE_HOME: home.path,
      XDG_DATA_HOME: home.path,
      XDG_RUNTIME_DIR: home.path,
      TMPDIR: home.path,
      PI_OFFLINE: '1',
    },
    stdio: [
      'ignore',
      'pipe',
      'pipe',
    ],
  },
);
/**
 Drain both pipes and keep the home alive until the child is closed.
 */
const [
  ,
  stdout,
  stderr,
] = await Promise.all([
  once(
    child,
    'close',
  ),
  text(child.stdout,),
  text(child.stderr,),
],);
if (child.exitCode !== 0)
  throw new Error(
    'Pi verifier did not exit successfully.',
    {
      cause: {
        exitCode: child.exitCode,
        signal: child.signalCode,
        stderr,
      },
    },
  );
process.stdout
  .write(stdout,);
process.stderr
  .write(stderr,);

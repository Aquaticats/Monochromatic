/**
 Run the real Pi SDK without inherited credentials, user config, or project extensions.
 @module
 */

import { execFile, } from 'node:child_process';
import { mkdtempDisposable, } from 'node:fs/promises';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';
import { promisify, } from 'node:util';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

/**
 Temporary HOME and working directory for a credential-free Pi process.
 */
await using home = await mkdtempDisposable(join(
  scratchRoot(),
  'pi-rate-limit-fallback-host-',
),);

/** Child result, isolated from the harness's credential environment and resource discovery. */
const { stdout, stderr, } = await promisify(execFile,)(
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
      HOME: home.path,
      XDG_CONFIG_HOME: home.path,
      XDG_CACHE_HOME: home.path,
      XDG_DATA_HOME: home.path,
      XDG_RUNTIME_DIR: home.path,
      TMPDIR: home.path,
      PI_OFFLINE: '1',
    },
  },
);
process.stdout.write(stdout,);
process.stderr.write(stderr,);

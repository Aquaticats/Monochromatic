/**
 Fault injection for the native TypeScript child, loaded into an Oxlint process with `node --import`.

 Makes every spawn of a `--api` child fail the way Node reports a spawn the operating system could not
 map memory for: a synchronous error whose code is `ENOMEM`. No memory is exhausted. Spawns of other
 programs pass through untouched.

 @module
 */

import childProcess from 'node:child_process';
import { syncBuiltinESMExports, } from 'node:module';

/**
 Spawns refused so far, reported at exit so a test can tell the injection was live.
 */
const refusals = { count: 0, };

/**
 Original spawn, reached for every program other than the native TypeScript child.
 */
const originalSpawn = childProcess.spawn;

/**
 Replacement for `spawn`: the native TypeScript child fails as an out-of-memory spawn does, and every other
 program starts as it would have. A proxy forwards the parameters exactly as `spawn` received them.
 */
childProcess.spawn = new Proxy(
  originalSpawn,
  {
    apply(
      target: typeof originalSpawn,
      receiver: unknown,
      parameters: Parameters<typeof originalSpawn>,
    ): unknown {
    /**
     Arguments handed to the program.
     */
      const [, args,] = parameters;
      if (Array.isArray(args,) && args.includes('--api',)) {
        refusals.count += 1;
        throw Object.assign(
          new Error('spawn ENOMEM',),
          {
            code: 'ENOMEM',
            errno: -12,
            syscall: 'spawn',
          },
        );
      }
      return Reflect.apply(
        target,
        receiver,
        parameters,
      );
    },
  },
);
syncBuiltinESMExports();
process.once(
  'exit',
  function reportRefusals(): void {
    process.stderr
      .write(`[refuse-native-api-spawn] refused ${String(refusals.count,)} native spawns\n`,);
  },
);

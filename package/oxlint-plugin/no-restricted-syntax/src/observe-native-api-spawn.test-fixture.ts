/**
 Observer for the native TypeScript child, loaded into a Node process with `node --import`.

 Passes every spawn through untouched and counts the ones that start the native TypeScript child, separating
 those that returned a child from those that threw. The counts are written to standard error at exit, so a
 test can tell whether, and whether successfully, the child was started.

 @module
 */

import childProcess from 'node:child_process';
import { syncBuiltinESMExports, } from 'node:module';

/**
 Native child spawns seen so far.
 */
const spawns = {
  started: 0,
  failed: 0,
};

/**
 Original spawn, reached for every program.
 */
const originalSpawn = childProcess.spawn;

/**
 Replacement for `spawn`: a proxy forwards the parameters exactly as `spawn` received them and counts the
 native TypeScript child by its `--api` argument.
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
      /**
       Whether this spawn starts the native TypeScript child.
       */
      const isNativeApi = Array.isArray(args,) && args.includes('--api',);
      try {
        /**
         Child process, or whatever the original returned.
         */
        const child = Reflect.apply(
          target,
          receiver,
          parameters,
        );
        if (isNativeApi)
          spawns.started += 1;
        return child;
      }
      catch (error: unknown) {
        if (isNativeApi)
          spawns.failed += 1;
        throw error;
      }
    },
  },
);
syncBuiltinESMExports();
process.once(
  'exit',
  function reportSpawns(): void {
    process.stderr
      .write(
      `[observe-native-api-spawn] started ${String(spawns.started,)} native spawns, failed ${String(spawns.failed,)}\n`,
      );
  },
);

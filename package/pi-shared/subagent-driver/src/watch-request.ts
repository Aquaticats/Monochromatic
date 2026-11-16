/**
 What watching a run needs, kept apart from the watch itself so the parts of it that answer a model error can name it without
 importing the loop.

 @module
 */

import type { ConnectionToPi, } from './connection-to-pi.ts';
import type {
  DriverOptions,
  RecordOfPiRun,
} from './driver-types.ts';

/**
 What watching a run needs.
 */
export type WatchRequest = {
  readonly connection: ConnectionToPi;
  readonly record: RecordOfPiRun;
  readonly options: DriverOptions;
  readonly inbox: string;
  readonly deadline: number;

  /**
   How many values of this machine's secret environment the driver handed Pi, when the chance to hand them over came.

   Pi reports whether the extension that masks them is loaded by registering a command whose description holds that count, so a
   run whose Pi comes back without it is a run whose tool results would reach a model unmasked. A caller that starts no Pi of its
   own, as the tests of the watch do, passes none and the question is not asked.
   */
  readonly secretValuesHandedOver?: number;

  /**
   The work this run left running, asked before the run may end as one that finished its turn.

   A run that ends its turn while a build it started still compiles is not a run with nothing to do, and a driver that ended
   one closed a session whose work was never read. Each line names one process, as the process's own tool recorded it; a caller
   that cannot look passes none, and the run ends as it always did.
   */
  readonly workLeftRunning?: () => Promise<readonly string[]>;
};

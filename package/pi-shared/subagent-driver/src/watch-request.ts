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
};

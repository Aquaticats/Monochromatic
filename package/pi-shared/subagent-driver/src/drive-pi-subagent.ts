#!/usr/bin/env node
/**
 `drive-pi-subagent`: runs one Pi agent unattended over RPC. See `README.md` for the options and the files a run writes.

 A run whose `--provider` and `--model` name a pool that takes no new run is refused before Pi starts and before anything reaches
 the run folder, so a caller that starts this command directly, past the pool gate, spends no slot on a closed pool.

 @module
 */

import {
  argv,
  env,
} from 'node:process';
import { parseDriverOptions, } from './parse-driver-options.ts';
import { PoolLimitsError, } from './pool-limits.ts';
import {
  CLOSED_POOL_EXIT_STATUS,
  ClosedPoolRefusalError,
  refuseARunOnAClosedPool,
} from './refusal-of-a-run-on-a-closed-pool.ts';
import { supervisePiRun, } from './supervise-pi-run.ts';

/**
 What this run was asked for, read before anything else: a run refused here wrote nothing at all.
 */
const options = parseDriverOptions(argv.slice(2,),);

try {
  refuseARunOnAClosedPool({
    provider: options.provider,
    model: options.model,
    environment: env,
  },);
  await supervisePiRun({
    options,
    environment: env,
  },);
}
catch (error) {
  // The refusal has a status of its own, so a caller tells a run that never started from one that started and failed.
  if (error instanceof ClosedPoolRefusalError) {
    console.error(error.message,);
    process.exitCode = CLOSED_POOL_EXIT_STATUS;
  }
  else if (error instanceof PoolLimitsError) {
    // A setting this command cannot read is the caller's own mistake, like a missing option: it is named with its fault and
    // without a stack, and no Pi is started. It is no closed pool, so it carries the status of any other failure.
    console.error(error.message,);
    process.exitCode = 1;
  }
  else {
    throw error;
  }
}

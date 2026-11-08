#!/usr/bin/env node
/**
 `drive-pi-subagent`: runs one Pi agent unattended over RPC. See `README.md` for the options and the files a run writes.

 @module
 */

import {
  argv,
  env,
} from 'node:process';
import { parseDriverOptions, } from './parse-driver-options.ts';
import { supervisePiRun, } from './supervise-pi-run.ts';

await supervisePiRun({
  options: parseDriverOptions(argv.slice(2,),),
  environment: env,
},);

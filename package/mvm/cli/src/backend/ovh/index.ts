/**
 OVHcloud Public Cloud backend.

 Assembles the lifecycle, exec, and update operations into a {@link Backend}.
 The registry loads this module lazily, so the OVHcloud code (and its key
 requirement) never engages unless the backend is selected.

 Every machine this backend makes is an OVHcloud Public Cloud instance on the
 hourly price, which the provider bills by the second after a first minute and
 until the instance is deleted; a stopped instance still bills. Nothing here ever
 asks for monthly billing.

 @module
 */

import type { Backend, } from '../types.ts';
import {
  ovhExec,
  ovhPull,
  ovhPush,
  ovhRun,
  ovhShell,
} from './exec.ts';
import {
  ovhClone,
  ovhCreate,
  ovhDestroy,
  ovhDestroyAll,
  ovhList,
} from './lifecycle.ts';
import { ovhUpdate, } from './update.ts';

/**
 OVHcloud Public Cloud backend implementing the full {@link Backend} contract.
 
 @example
 ```ts
 await ovhBackend.create({ name: 'dev-01' });
 await ovhBackend.exec({ command: 'uname -a', name: 'dev-01' });
 ```
 */
export const ovhBackend: Backend = {
  clone: ovhClone,
  create: ovhCreate,
  destroy: ovhDestroy,
  destroyAll: ovhDestroyAll,
  exec: ovhExec,
  list: ovhList,
  pullFile: ovhPull,
  pushFile: ovhPush,
  run: ovhRun,
  shell: ovhShell,
  update: ovhUpdate,
};

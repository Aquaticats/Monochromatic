/**
 Hetzner Cloud backend.
 
 Assembles the lifecycle, exec, and update operations into a {@link Backend}.
 The registry loads this module lazily so the Hetzner code (and a token
 requirement) never engages unless the backend is selected.
 
 @module
 */

import type { Backend, } from '../types.ts';
import {
  hetznerExec,
  hetznerPull,
  hetznerPush,
  hetznerRun,
  hetznerShell,
} from './exec.ts';
import {
  hetznerClone,
  hetznerCreate,
  hetznerDestroy,
  hetznerDestroyAll,
  hetznerList,
} from './lifecycle.ts';
import { hetznerUpdate, } from './update.ts';

/**
 Hetzner Cloud backend implementing the full {@link Backend} contract.
 
 @example
 ```ts
 await hetznerBackend.create({ name: 'dev-01' });
 await hetznerBackend.exec({ command: 'uname -a', name: 'dev-01' });
 ```
 */
export const hetznerBackend: Backend = {
  clone: hetznerClone,
  create: hetznerCreate,
  destroy: hetznerDestroy,
  destroyAll: hetznerDestroyAll,
  exec: hetznerExec,
  list: hetznerList,
  pullFile: hetznerPull,
  pushFile: hetznerPush,
  run: hetznerRun,
  shell: hetznerShell,
  update: hetznerUpdate,
};

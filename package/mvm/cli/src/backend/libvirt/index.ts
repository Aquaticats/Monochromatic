/**
 Local QEMU/KVM backend.
 
 Wraps the existing standalone libvirt functions into a {@link Backend} object
 without changing their behaviour. This is the default backend and the only
 host-local one today.
 
 @module
 */

import { clone, } from '../../clone.ts';
import { create, } from '../../create.ts';
import {
  destroy,
  destroyAll,
} from '../../destroy.ts';
import { exec, } from '../../exec.ts';
import {
  pullFile,
  pushFile,
} from '../../file-transfer.ts';
import { list, } from '../../list.ts';
import { run, } from '../../run.ts';
import { shell, } from '../../shell.ts';
import type { Backend, } from '../types.ts';
import { update, } from '../../update.ts';

/**
 Local libvirt/KVM backend assembled from the package's standalone functions.
 The cloud-only `serverType`/`location` hints on {@link create} are ignored here.
 
 @example
 ```ts
 await libvirtBackend.create({ name: 'dev-01' });
 await libvirtBackend.exec({ command: 'uname -a', name: 'dev-01' });
 ```
 */
export const libvirtBackend: Backend = {
  clone,
  create,
  destroy,
  destroyAll,
  exec,
  list,
  pullFile,
  pushFile,
  run,
  shell,
  update,
};

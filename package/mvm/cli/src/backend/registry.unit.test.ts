/**
 Unit tests for the backend registry: kind resolution, the pure platform
 guard, and lazy selection. No network or filesystem.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isBackendAvailable,
  resolveBackendKind,
  selectBackend,
} from '@monochromatic-dev/mvm-cli/ts/backend/registry.ts';

/**
 Sets an env var for the duration of a `using` scope, restoring it after.
 */
function withEnv(key: string, value?: string,): Disposable {
  const prior = process.env[key];
  if (value === undefined) {
    Reflect.deleteProperty(process.env, key,);
  }
  else {
    process.env[key] = value;
  }
  return {
    [Symbol.dispose]() {
      if (prior === undefined) {
        Reflect.deleteProperty(process.env, key,);
      }
      else {
        process.env[key] = prior;
      }
    },
  };
}

/**
 Overrides `process.platform` for the duration of a `using` scope.
 */
function withPlatform(platform: string,): Disposable {
  const original = Object.getOwnPropertyDescriptor(process, 'platform',);
  Object.defineProperty(process, 'platform', { configurable: true, value: platform, },);
  return {
    [Symbol.dispose]() {
      if (original !== undefined) {
        Object.defineProperty(process, 'platform', original,);
      }
    },
  };
}

await describe({
  name: 'hetzner backend registry',
  concurrency: 1,
  children: [
    it({
      name: 'resolveBackendKind defaults to libvirt with no flag or env',
      fn: async () => {
        using _env = withEnv('MVM_BACKEND', undefined,);
        expect(resolveBackendKind(),).toBe('libvirt',);
      },
    },),
    it({
      name: 'resolveBackendKind returns the explicit kind',
      fn: async () => {
        using _env = withEnv('MVM_BACKEND', undefined,);
        expect(resolveBackendKind('hetzner',),).toBe('hetzner',);
        expect(resolveBackendKind('libvirt',),).toBe('libvirt',);
      },
    },),
    it({
      name: 'resolveBackendKind falls through an empty flag to MVM_BACKEND',
      fn: async () => {
        using _env = withEnv('MVM_BACKEND', 'hetzner',);
        expect(resolveBackendKind('',),).toBe('hetzner',);
      },
    },),
    it({
      name: 'resolveBackendKind lets an explicit flag override the env',
      fn: async () => {
        using _env = withEnv('MVM_BACKEND', 'hetzner',);
        expect(resolveBackendKind('libvirt',),).toBe('libvirt',);
      },
    },),
    it({
      name: 'resolveBackendKind throws on an unknown backend',
      fn: async () => {
        using _env = withEnv('MVM_BACKEND', undefined,);
        expect(() => resolveBackendKind('aws',),).toThrow('unknown backend',);
      },
    },),
    it({
      name: 'isBackendAvailable gates libvirt to linux',
      fn: async () => {
        expect(isBackendAvailable({ kind: 'libvirt', platform: 'linux', },),).toBe(true,);
        expect(isBackendAvailable({ kind: 'libvirt', platform: 'win32', },),).toBe(false,);
        expect(isBackendAvailable({ kind: 'libvirt', platform: 'darwin', },),).toBe(false,);
      },
    },),
    it({
      name: 'isBackendAvailable allows hetzner on every platform',
      fn: async () => {
        expect(isBackendAvailable({ kind: 'hetzner', platform: 'linux', },),).toBe(true,);
        expect(isBackendAvailable({ kind: 'hetzner', platform: 'win32', },),).toBe(true,);
        expect(isBackendAvailable({ kind: 'hetzner', platform: 'darwin', },),).toBe(true,);
      },
    },),
    it({
      name: 'selectBackend loads the hetzner backend on any platform',
      fn: async () => {
        using _platform = withPlatform('win32',);
        const backend = await selectBackend('hetzner',);
        expect(typeof backend.create,).toBe('function',);
        expect(typeof backend.exec,).toBe('function',);
      },
    },),
    it({
      name: 'selectBackend throws for libvirt on a non-linux platform',
      fn: async () => {
        using _platform = withPlatform('win32',);
        expect(() => selectBackend('libvirt',),).toThrow('not available',);
      },
    },),
  ],
},);

/**
 Covers every branch of `prepareOptions`:
 filled defaults,
 caller overrides,
 file-extension normalization,
 encryption-algorithm validation,
 and config-directory resolution.
 
 @module
 */

import {
  basename,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  InvalidEncryptionAlgorithmError,
  MissingProjectNameError,
  prepareOptions,
} from '../dist/final/neutral/index.mjs';

import {
  createDisposableHome,
  restoreHome,
} from './test-support.ts';

/**
 Runs a call expected to throw and returns the captured error so class and message text can be asserted.
 
 @param call - Call that must throw.
 
 @returns The captured thrown value.
 */
function captureThrown(call: () => unknown,): Error {
  try {
    call();
  }
  catch (error) {
    return error as Error;
  }
  throw new Error('expected the call to throw, but it returned',);
}

await describe({
  name: prepareOptions.name,
  children: [
    it({
      name: 'fills upstream defaults when the caller passes only a cwd',
      fn: async () => {
        /**
         Prepared options carrying every default knob.
         */
        const prepared = prepareOptions({ cwd: '/tmp/app', },);
        expect(prepared.configName,).toBe('config',);
        expect(prepared.fileExtension,).toBe('json',);
        expect(prepared.projectSuffix,).toBe('nodejs',);
        expect(prepared.clearInvalidConfig,).toBe(false,);
        expect(prepared.accessPropertiesByDotNotation,).toBe(true,);
        expect(prepared.configFileMode,).toBe(0o666,);
        expect(prepared.encryptionAlgorithm,).toBe('aes-256-cbc',);
      },
    },),
    it({
      name: 'lets every caller override win over the default',
      fn: async () => {
        /**
         Prepared options carrying an override for every defaulted knob.
         */
        const prepared = prepareOptions({
          cwd: '/tmp/app',
          configName: 'settings',
          fileExtension: 'json5',
          projectSuffix: 'desktop',
          clearInvalidConfig: true,
          accessPropertiesByDotNotation: false,
          configFileMode: 0o600,
          encryptionAlgorithm: 'aes-256-gcm',
        },);
        expect(prepared.configName,).toBe('settings',);
        expect(prepared.fileExtension,).toBe('json5',);
        expect(prepared.projectSuffix,).toBe('desktop',);
        expect(prepared.clearInvalidConfig,).toBe(true,);
        expect(prepared.accessPropertiesByDotNotation,).toBe(false,);
        expect(prepared.configFileMode,).toBe(0o600,);
        expect(prepared.encryptionAlgorithm,).toBe('aes-256-gcm',);
      },
    },),
    it({
      name: 'keeps an explicitly empty projectSuffix when resolving the config directory',
      fn: async () => {
        /**
         Prepared options whose directory keeps the bare project name.
         */
        const prepared = prepareOptions({
          projectName: 'myapp',
          projectSuffix: '',
        },);
        expect(prepared.projectSuffix,).toBe('',);
        expect(basename(prepared.cwd,),).toBe('myapp',);
      },
    },),
    it({
      name: 'resolves cwd through the config directory when only projectName is set',
      fn: async () => {
        /**
         Disposable home the config directory must resolve inside.
         */
        const home = createDisposableHome();
        /**
         Restore the injected home environment when the test ends.
         */
        using restore = {
          [Symbol.dispose](): void {
            restoreHome(home,);
          },
        };
        /**
         Prepared options whose cwd resolves through the home-based config directory.
         */
        const prepared = prepareOptions({ projectName: 'myapp', },);
        expect(prepared.cwd.endsWith('myapp-nodejs',),).toBe(true,);
        expect(basename(prepared.cwd,),).toBe('myapp-nodejs',);
        expect(prepared.cwd.startsWith(home.home,),).toBe(true,);
      },
    },),
    it({
      name: 'skips projectName resolution when cwd is present',
      fn: async () => {
        /**
         Prepared options keeping the caller-supplied cwd verbatim.
         */
        const prepared = prepareOptions({
          cwd: '/tmp/app',
          projectName: '',
        },);
        expect(prepared.cwd,).toBe('/tmp/app',);
      },
    },),
    it({
      name: 'strips a single leading dot from the file extension',
      fn: async () => {
        /**
         Prepared options normalized from a `.json`-style extension.
         */
        const prepared = prepareOptions({
          cwd: '/tmp/app',
          fileExtension: '.json',
        },);
        expect(prepared.fileExtension,).toBe('json',);
      },
    },),
    it({
      name: 'strips repeated leading dots from the file extension',
      fn: async () => {
        /**
         Prepared options normalized from an `..json`-style extension.
         */
        const prepared = prepareOptions({
          cwd: '/tmp/app',
          fileExtension: '..json',
        },);
        expect(prepared.fileExtension,).toBe('json',);
      },
    },),
    it({
      name: 'leaves an empty file extension empty',
      fn: async () => {
        /**
         Prepared options carrying the empty extension.
         */
        const prepared = prepareOptions({
          cwd: '/tmp/app',
          fileExtension: '',
        },);
        expect(prepared.fileExtension,).toBe('',);
      },
    },),
    it({
      name: 'passes a non-string file extension through untouched',
      fn: async () => {
        /**
         Prepared options whose extension is not a string at runtime.
         */
        const prepared = prepareOptions({
          cwd: '/tmp/app',
          fileExtension: 42 as never,
        },);
        expect(prepared.fileExtension,).toBe(42,);
      },
    },),
    it({
      name: 'throws InvalidEncryptionAlgorithmError listing the supported set for an unsupported name',
      fn: async () => {
        /**
         Error thrown while rejecting the unsupported algorithm name.
         */
        const error = captureThrown((): unknown => prepareOptions({
          cwd: '/tmp/app',
          encryptionAlgorithm: 'rot13' as never,
        },),);
        expect(error,).toBeInstanceOf(InvalidEncryptionAlgorithmError,);
        expect(error.name,).toBe('InvalidEncryptionAlgorithmError',);
        expect(error.message,).toBe('The `encryptionAlgorithm` option must be one of: aes-256-cbc, aes-256-gcm, aes-256-ctr',);
      },
    },),
    it({
      name: 'throws InvalidEncryptionAlgorithmError for a non-string algorithm name',
      fn: async () => {
        /**
         Error thrown while rejecting the non-string algorithm value.
         */
        const error = captureThrown((): unknown => prepareOptions({
          cwd: '/tmp/app',
          encryptionAlgorithm: 42 as never,
        },),);
        expect(error,).toBeInstanceOf(InvalidEncryptionAlgorithmError,);
      },
    },),
    it({
      name: 'falls back to aes-256-cbc when encryptionAlgorithm is nullish',
      fn: async () => {
        /**
         Prepared options whose nullish algorithm resolves to the default.
         */
        const prepared = prepareOptions({
          cwd: '/tmp/app',
          encryptionAlgorithm: null as never,
        },);
        expect(prepared.encryptionAlgorithm,).toBe('aes-256-cbc',);
      },
    },),
    it({
      name: 'throws MissingProjectNameError when neither cwd nor projectName is set',
      fn: async () => {
        /**
         Error thrown while no directory input resolves.
         */
        const error = captureThrown((): unknown => prepareOptions({},),);
        expect(error,).toBeInstanceOf(MissingProjectNameError,);
        expect(error.name,).toBe('MissingProjectNameError',);
        expect(error.message,).toBe('Please specify the `projectName` option.',);
      },
    },),
    it({
      name: 'throws MissingProjectNameError for an empty-string projectName without a cwd',
      fn: async () => {
        /**
         Error thrown for an empty projectName with no cwd at all.
         */
        const errorWithoutCwd = captureThrown((): unknown => prepareOptions({ projectName: '', },),);
        expect(errorWithoutCwd,).toBeInstanceOf(MissingProjectNameError,);
        /**
         Error thrown for an empty projectName with an empty-string cwd.
         */
        const errorWithEmptyCwd = captureThrown((): unknown => prepareOptions({
          cwd: '',
          projectName: '',
        },),);
        expect(errorWithEmptyCwd,).toBeInstanceOf(MissingProjectNameError,);
      },
    },),
    it({
      name: 'treats an empty-string cwd like an absent cwd when projectName is set',
      fn: async () => {
        /**
         Disposable home the config directory must resolve inside.
         */
        const home = createDisposableHome();
        /**
         Restore the injected home environment when the test ends.
         */
        using restore = {
          [Symbol.dispose](): void {
            restoreHome(home,);
          },
        };
        /**
         Prepared options resolved through the home-based config directory despite the empty cwd.
         */
        const prepared = prepareOptions({
          cwd: '',
          projectName: 'myapp',
        },);
        expect(prepared.cwd.endsWith('myapp-nodejs',),).toBe(true,);
        expect(prepared.cwd.startsWith(home.home,),).toBe(true,);
      },
    },),
  ],
},);

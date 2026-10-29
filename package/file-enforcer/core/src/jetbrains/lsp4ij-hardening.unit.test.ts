import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  l,
  manageLsp4ijServerSettings,
} from '../../dist/final/node/index.mjs';

/**
 Minimal LSP4IJ settings needed to exercise discovery failure handling.
 */
const SETTINGS = {
  productPrefixes: ['IntelliJIdea',],
  baseServerMatch: {
    templateId: 'harper-ls',
  },
  scopedServers: [],
} as const;

//region Environment and fixture helpers

/**
 Sets `XDG_CONFIG_HOME` for scoped JetBrains discovery tests.
 
 @param configRoot - Directory exposed as XDG config root.
 
 @returns Disposable that restores prior environment state.
 
 @example
 ```ts
 using xdg = withXdgConfigHome('/tmp/config');
 ```
 */
function withXdgConfigHome(configRoot: string,): Disposable {
  /**
   Prior XDG config root, restored on disposal.
   */
  const previous = process.env.XDG_CONFIG_HOME;
  process.env.XDG_CONFIG_HOME = configRoot;
  return {
    [Symbol.dispose](): void {
      if (previous === undefined)
        delete process.env.XDG_CONFIG_HOME;
      else
        process.env.XDG_CONFIG_HOME = previous;
    },
  };
}

/**
 Owns temporary directory cleanup for JetBrains LSP4IJ hardening tests.
 
 @param directory - Directory to remove on disposal.
 
 @returns Async disposable that removes directory recursively.
 
 @example
 ```ts
 await using owned = throwawayDir('/tmp/config');
 ```
 */
function throwawayDir(directory: string,): AsyncDisposable {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        directory,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

//endregion Environment and fixture helpers

//region Error capture helpers

/**
 Captures LSP4IJ sync failure for assertion.
 
 @returns Caught error, or undefined when sync resolves.
 
 @example
 ```ts
 const error = await manageLsp4ijServerSettingsError();
 ```
 */
async function manageLsp4ijServerSettingsError(): Promise<unknown> {
  try {
    await manageLsp4ijServerSettings({ settings: SETTINGS, },);
  }
  catch (error: unknown) {
    return error;
  }

  return undefined;
}

//endregion Error capture helpers

await describe({
  name: manageLsp4ijServerSettings.name,
  // Serialized because the suite mutates shared process.env.XDG_CONFIG_HOME and stubs shared logger state.
  concurrency: 1,
  children: [
    it({
      name: 'propagates corrupt JetBrains config root read errors',
      fn: async function propagatesCorruptConfigRoot({ sinon, },): Promise<void> {
        const warnStub = sinon.stub(l, 'warn',);
        /**
         Throwaway XDG root whose JetBrains entry is a file, not directory.
         */
        const configRoot = await mkdtemp(join(
          tmpdir(),
          'fe-lsp4ij-hardening-',
        ),);
        await using owned = throwawayDir(configRoot,);
        using xdg = withXdgConfigHome(configRoot,);
        await writeFile(
          join(
            configRoot,
            'JetBrains',
          ),
          'not a directory',
        );

        const caught = await manageLsp4ijServerSettingsError();
        expect(caught,).toMatchObject({
          code: 'ENOTDIR',
        },);
        expect(warnStub,).not.toHaveBeenCalled();
        void owned;
        void xdg;
      },
    },),
  ],
},);

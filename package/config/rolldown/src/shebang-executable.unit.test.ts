import {
  mkdtempSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { shebangExecutablePlugin, } from './shebang-executable.ts';

/**
 Owner-executable permission probe mask.
 */
const OWNER_EXECUTE = 0o100;

/**
 Disposable output-directory fixture.
 */
type OutputFixture = {
  readonly path: string;
  [Symbol.dispose]: () => void;
};

/**
 Creates a disposable output directory holding pre-written chunk files.
 
 @returns Disposable fixture outside repository state.
 
 @example
 ```ts
 using fixture = createOutputFixture();
 ```
 */
function createOutputFixture(): OutputFixture {
  /**
   Fresh fixture directory under the system temp root.
   */
  const path = mkdtempSync(join(scratchRoot(), 'shebang-executable-',),);
  return {
    path,
    [Symbol.dispose]: function removeFixture(): void {
      rmSync(path, {
        force: true,
        recursive: true,
      },);
    },
  };
}

await describe({
  name: shebangExecutablePlugin.name,
  children: [
    it({
      name: 'marks shebang chunks executable and leaves plain chunks alone',
      fn: async function chmodBehavior(): Promise<void> {
        using fixture = createOutputFixture();
        writeFileSync(
          join(fixture.path, 'cli.mjs',),
          '#!/usr/bin/env node\nconsole.log(1);\n',
          { mode: 0o644, },
        );
        writeFileSync(
          join(fixture.path, 'lib.mjs',),
          'export const x = 1;\n',
          { mode: 0o644, },
        );
        /**
         Plugin under test.
         */
        const plugin = shebangExecutablePlugin();
        /**
         writeBundle hook extracted for direct invocation.
         */
        const writeBundle = plugin.writeBundle as (
          options: { readonly dir?: string; },
          bundle: Readonly<Record<string, {
            readonly type: string;
            readonly code: string;
          }>>,
        ) => Promise<void>;
        await writeBundle(
          { dir: fixture.path, },
          {
            'cli.mjs': {
              type: 'chunk',
              code: '#!/usr/bin/env node\nconsole.log(1);\n',
            },
            'lib.mjs': {
              type: 'chunk',
              code: 'export const x = 1;\n',
            },
          },
        );
        expect((statSync(join(fixture.path, 'cli.mjs',),).mode & OWNER_EXECUTE) !== 0,).toBe(true,);
        expect((statSync(join(fixture.path, 'lib.mjs',),).mode & OWNER_EXECUTE) !== 0,).toBe(false,);
      },
    },),
  ],
},);

import { access, } from 'node:fs/promises';
import {
  dirname,
  join,
  resolve,
} from 'node:path';
import { pathToFileURL, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { stageWgQuickerCli, } from './tunnel-bypass-command-fixture.ts';

/**
 Resolver module shape loaded from the staged copy.
 */
type ResolverModule = {
  readonly resolveApplicationExemptionCommand: () => Promise<string>;
};

await describe({
  name: stageWgQuickerCli.name,
  children: [
    it({
      name: 'makes the staged CLI find the stand-in watcher next to its own package, with no environment variable involved',
      fn: async () => {
        /**
         Staged tree that is removed at the end of the test.
         */
        const staged = await stageWgQuickerCli();
        /**
         Directory of the staged bundle.
         */
        const bundleDirectory = dirname(staged.cliPath,);
        /**
         Stand-in watcher the staged CLI must resolve:
         the release path next to the staged package.
         */
        const standIn = join(
          resolve(
            bundleDirectory,
            '..',
            '..',
            '..',
            '..',
          ),
          'cli-quicker-exempt',
          'target',
          'release',
          'wg-quicker-exempt',
        );
        /**
         Resolver of the staged copy.
         */
        const resolver = await import(pathToFileURL(join(bundleDirectory, 'application-exemption-command.mjs',),).href,) as ResolverModule;
        expect(await resolver.resolveApplicationExemptionCommand(),).toBe(standIn,);
        await staged[Symbol.asyncDispose]();
        await expect(access(staged.cliPath,),).rejects.toThrow();
      },
    },),
  ],
},);

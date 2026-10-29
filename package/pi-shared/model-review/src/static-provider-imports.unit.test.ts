import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 Production provider dispatch source.
 */
const PROVIDER_STREAMS_SOURCE = new URL(
  'provider-streams.ts',
  import.meta.url,
);

await describe({
  name: 'static provider imports',
  children: [
    it({
      name: 'uses only static Pi lazy provider modules',
      fn: async () => {
        /**
         Authored provider dispatch source text.
         */
        const source = await readFile(
          PROVIDER_STREAMS_SOURCE,
          'utf8',
        );
        expect(source.includes('import(',),).toBe(false,);
        /**
         Pi provider API import lines.
         */
        const providerImportLines = source
          .split('\n',)
          .filter(function providerImportLine(line,): boolean {
            return line.includes("from '@earendil-works/pi-ai/api/",);
          },);
        expect(providerImportLines.length > 0,).toBe(true,);
        expect(providerImportLines.every(function lazyProviderImport(line,): boolean {
          return line.includes(".lazy';",);
        },),).toBe(true,);
      },
    },),
  ],
},);

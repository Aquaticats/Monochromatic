/**
 Tests for the providers the meter report reads as left off (ledger B1209).

 A provider the run leaves off has no client, so its meter reads dry with nothing to report on every reading, which the
 report once printed as an outage. The launch and the budget sample write a `PROVIDER OFF <provider>: <reason>` line for
 each provider they leave off, and the report reads it from the same logs as the readings.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  leftOffLines,
  providersLeftOffIn,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'meter report off',
  concurrency: 1,
  children: [
    describe({
      name: providersLeftOffIn.name,
      concurrency: 1,
      children: [
        it({
          name: 'READS THE PROVIDER AND THE REASON OF A PROVIDER OFF LINE, whatever the log puts before it',
          fn: async () => {
            expect(providersLeftOffIn({
              texts: ['[info] [2026-08-25T09:59:00.000Z] [configureProviders] PROVIDER OFF bedrock: the owner left it off\n',],
            },),).toEqual([{ provider: 'bedrock', reason: 'the owner left it off', },],);
          },
        },),

        it({
          name: 'NAMES EACH PROVIDER ONCE, with the reason of the first line that names it, across every log',
          fn: async () => {
            expect(providersLeftOffIn({
              texts: [
                'PROVIDER OFF hyper: first reason\nPROVIDER OFF hyper: second reason\n',
                'PROVIDER OFF bedrock: the owner left it off\n',
              ],
            },),).toEqual([
              { provider: 'hyper', reason: 'first reason', },
              { provider: 'bedrock', reason: 'the owner left it off', },
            ],);
          },
        },),

        it({
          name: 'READS NO PROVIDER FROM A WORD THAT IS NONE, A LINE WITH NO COLON, OR A LOG WITHOUT THE MARKER',
          fn: async () => {
            expect(providersLeftOffIn({
              texts: ['PROVIDER OFF the-cat-flap: nobody\nPROVIDER OFF bedrock without a colon\nan ordinary line\n',],
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: leftOffLines.name,
      concurrency: 1,
      children: [
        it({
          name: 'SAYS THE PROVIDER IS LEFT OFF AND GIVES IT NO FIGURES',
          fn: async () => {
            expect(leftOffLines({ off: { provider: 'bedrock', reason: 'the owner left it off', }, },),).toEqual([
              '\nbedrock: LEFT OFF, not asked (PROVIDER OFF line: the owner left it off)',
              '  its meter reads dry with nothing to report, so no wet or dry count, spendable share or outage is given '
                + 'for it',
            ],);
          },
        },),
      ],
    },),
  ],
},);

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  checkExitConsistency,
  extractPolicyEvents,
  type PolicyEvent,
} from './jsonl-event-fixture.ts';

//region Fixtures

/**
 Serializes one event line.

 @param event - event fields

 @returns JSONL line

 @example
 ```ts
 line({ sequence: 0, type: 'finding' });
 ```
 */
function line(event: Readonly<Record<string, unknown>>,): string {
  return JSON.stringify({ schemaVersion: 1, ...event, },);
}

/**
 Error-severity finding.
 */
const ERROR_FINDING: PolicyEvent = { sequence: 0, type: 'finding', severity: 'error', code: 'x/y', };

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: extractPolicyEvents.name,
      children: [
        it({
          name: 'keeps events between real Git output lines and ignores trust warnings',
          fn: async () => {
            const extraction = extractPolicyEvents([
              'fatal: something human',
              line({ sequence: 0, type: 'finding', severity: 'warn', code: 'final-newline/missing', },),
              line({ type: 'trust-warning', code: 'relaxed-entry-malformed', },),
              line({ sequence: 1, type: 'commit-landed', oid: 'a'.repeat(40,), },),
              '',
            ].join('\n',),);
            expect(extraction.issues,).toEqual([],);
            expect(extraction.events,).toEqual([
              { sequence: 0, type: 'finding', severity: 'warn', code: 'final-newline/missing', },
              { sequence: 1, type: 'commit-landed', oid: 'a'.repeat(40,), },
            ],);
          },
        },),
        it({
          name: 'reports malformed lines, missing schema, and sequence gaps',
          fn: async () => {
            const extraction = extractPolicyEvents([
              '{not json',
              JSON.stringify({ sequence: 0, type: 'finding', },),
              '{"schemaVersion":1,"type":"finding"}',
              line({ sequence: 3, type: 'finding', },),
              '[1]',
            ].join('\n',),);
            expect(extraction.events,).toHaveLength(1,);
            expect(extraction.issues,).toHaveLength(4,);
            expect(extraction.issues.at(-1,),).toBe('event 0 has sequence 3',);
          },
        },),
      ],
    },),
    describe({
      name: checkExitConsistency.name,
      children: [
        it({
          name: 'accepts the documented exit contract',
          fn: async () => {
            expect(checkExitConsistency({ exitCode: 0, events: [], },),).toEqual([],);
            expect(checkExitConsistency({ exitCode: 0, events: [{ sequence: 0, type: 'finding', severity: 'warn', },], },),).toEqual([],);
            expect(checkExitConsistency({ exitCode: 1, events: [ERROR_FINDING,], },),).toEqual([],);
            expect(checkExitConsistency({ exitCode: 1, events: [{ sequence: 0, type: 'core-finding', },], },),).toEqual([],);
            expect(checkExitConsistency({ exitCode: 2, events: [{ sequence: 0, type: 'engine-failure', },], },),).toEqual([],);
            expect(checkExitConsistency({
              exitCode: 2,
              events: [ERROR_FINDING, { sequence: 1, type: 'commit-landed', oid: 'a', },],
            },),).toEqual([],);
            // Real Git's own failure codes pass through without events.
            expect(checkExitConsistency({ exitCode: 128, events: [], },),).toEqual([],);
          },
        },),
        it({
          name: 'reports every contract breach',
          fn: async () => {
            expect(checkExitConsistency({ exitCode: 0, events: [ERROR_FINDING,], },),)
              .toEqual(['blocking finding with exit 0', 'exit 0 with a blocking event',],);
            expect(checkExitConsistency({ exitCode: 1, events: [{ sequence: 0, type: 'engine-failure', },], },),)
              .toEqual(['engine-failure event with exit 1',],);
            expect(checkExitConsistency({
              exitCode: 0,
              events: [{ sequence: 0, type: 'commit-landed', oid: 'a', }, { sequence: 1, type: 'finding', severity: 'warn', },],
            },),).toEqual(['commit-landed event with exit 0', 'commit-landed is not the final event', 'exit 0 with a blocking event',],);
          },
        },),
      ],
    },),
  ],
},);

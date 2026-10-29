/**
 Tests for lone-Escape gating in the built bash-poke artifact.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  constants,
  createEscapeListener,
  isLoneEscape,
  shouldCancelOnEscape,
  type EscapeGate,
} from '../dist/final/node/index.mjs';

/**
 Builds a gate whose every condition allows cancellation.
 
 @param overrides - conditions to change for one case
 
 @returns gate ready for a decision
 
 @example
 ```ts
 gateWith({ jobCount: () => 0, });
 ```
 */
function gateWith(overrides: {
  readonly isIdle?: () => boolean;
  readonly editorText?: () => string;
  readonly jobCount?: () => number;
} = {}, ): EscapeGate {
  return {
    isIdle: () => true,
    editorText: () => '',
    jobCount: () => 1,
    ...overrides,
  };
}

await describe({
  name: '',
  children: [
    //region isLoneEscape

    describe({
      name: isLoneEscape.name,
      children: [
        it({
          name: 'accepts the bare escape byte',
          fn: async () => {
            expect(isLoneEscape({ data: constants.BARE_ESCAPE, }, ), ).toBe(true);
          },
        }, ),
        it({
          name: 'accepts the Kitty functional escape',
          fn: async () => {
            expect(isLoneEscape({ data: constants.KITTY_ESCAPE, }, ), ).toBe(true);
          },
        }, ),
        it({
          name: 'rejects an escape-prefixed combination',
          fn: async () => {
            expect(isLoneEscape({ data: `${constants.BARE_ESCAPE}k`, }, ), ).toBe(false);
          },
        }, ),
        it({
          name: 'rejects an arrow key sequence',
          fn: async () => {
            expect(isLoneEscape({ data: `${constants.BARE_ESCAPE}[A`, }, ), ).toBe(false);
          },
        }, ),
        it({
          name: 'rejects ordinary text and empty input',
          fn: async () => {
            expect(isLoneEscape({ data: 'a', }, ), ).toBe(false);
            expect(isLoneEscape({ data: '', }, ), ).toBe(false);
          },
        }, ),
      ],
    }, ),

    //endregion isLoneEscape

    //region shouldCancelOnEscape

    describe({
      name: shouldCancelOnEscape.name,
      children: [
        it({
          name: 'claims a lone escape while idle with jobs running',
          fn: async () => {
            expect(
              shouldCancelOnEscape({ data: constants.BARE_ESCAPE, gate: gateWith(), }, ),
            ).toBe(true);
          },
        }, ),
        it({
          name: 'leaves non-escape input alone',
          fn: async () => {
            expect(shouldCancelOnEscape({ data: 'x', gate: gateWith(), }, ), ).toBe(false);
          },
        }, ),
        it({
          name: 'leaves escape alone when no job is running',
          fn: async () => {
            expect(
              shouldCancelOnEscape({
                data: constants.BARE_ESCAPE,
                gate: gateWith({ jobCount: () => 0, }, ),
              }, ),
            ).toBe(false);
          },
        }, ),
        it({
          name: 'leaves escape alone while the agent is streaming',
          fn: async () => {
            expect(
              shouldCancelOnEscape({
                data: constants.BARE_ESCAPE,
                gate: gateWith({ isIdle: () => false, }, ),
              }, ),
            ).toBe(false);
          },
        }, ),
        it({
          name: 'leaves escape alone while Pi is in bash mode',
          fn: async () => {
            expect(
              shouldCancelOnEscape({
                data: constants.BARE_ESCAPE,
                gate: gateWith({ editorText: () => '! make', }, ),
              }, ),
            ).toBe(false);
          },
        }, ),
        it({
          name: 'claims the Kitty escape under the same conditions',
          fn: async () => {
            expect(
              shouldCancelOnEscape({ data: constants.KITTY_ESCAPE, gate: gateWith(), }, ),
            ).toBe(true);
          },
        }, ),
      ],
    }, ),

    //endregion shouldCancelOnEscape

    //region createEscapeListener

    describe({
      name: createEscapeListener.name,
      children: [
        it({
          name: 'cancels on a gated lone escape without consuming it',
          fn: async () => {
            const calls: number[] = [];
            const listener = createEscapeListener({
              gate: gateWith(),
              onCancel(): void {
                calls.push(1, );
              },
            }, );
            expect(listener(constants.BARE_ESCAPE, ), ).toBeUndefined();
            expect(calls, ).toHaveLength(1);
          },
        }, ),
        it({
          name: 'passes ordinary input through untouched',
          fn: async () => {
            const calls: number[] = [];
            const listener = createEscapeListener({
              gate: gateWith(),
              onCancel(): void {
                calls.push(1, );
              },
            }, );
            expect(listener('hello', ), ).toBeUndefined();
            expect(calls, ).toHaveLength(0);
          },
        }, ),
        it({
          name: 'passes escape through while the agent streams',
          fn: async () => {
            const calls: number[] = [];
            const listener = createEscapeListener({
              gate: gateWith({ isIdle: () => false, }, ),
              onCancel(): void {
                calls.push(1, );
              },
            }, );
            expect(listener(constants.BARE_ESCAPE, ), ).toBeUndefined();
            expect(calls, ).toHaveLength(0);
          },
        }, ),
      ],
    }, ),

    //endregion createEscapeListener
  ],
}, );

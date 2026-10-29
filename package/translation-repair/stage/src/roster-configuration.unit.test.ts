/**
 Tests for the configuration check both lanes run before they start.

 FOUND BY FAULT INJECTION: configuring zero critic models ran the repair lane
 end to end and returned a settled, unchanged document. No throw, no finding,
 no marker, and zero model exchanges bought. Downstream that is
 indistinguishable from a page that genuinely needed no repair, so a corpus
 pass under the misconfiguration would spend hours writing a directory of
 vacuous artifacts that later analysis reads as clean runs.

 The quiet path is RIGHT for outages, and stays. What this refuses is the
 deterministic case, before any work is done.

 Fixtures are invented. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  assertRostersConfigured,
  RosterConfigurationError,
} from '../dist/final/node/index.mjs';

/**
 Runs one configuration and returns whatever it raised.

 @param roles - rosters to check

 @returns Failure raised, or undefined when the configuration was accepted

 @example
 ```ts
 const caught = configurationFailure({ roles: { judgeModelIds: [], }, },);
 ```
 */
function configurationFailure(
  { roles, }: { readonly roles: Readonly<Record<string, readonly string[]>>; },
): unknown {
  try {
    assertRostersConfigured({
      lane: 'repair',
      roles,
    },);
  }
  catch (error) {
    return error;
  }
  return undefined;
}

await describe({
  name: assertRostersConfigured.name,
  children: [
    it({
      name: 'accepts a lane where every required role has somebody in it, which is the ordinary '
        + 'case and the one a configuration check must not tax',
      fn: async () => {
        expect(configurationFailure({
          roles: {
            editorModelIds: ['hf:one',],
            judgeModelIds: [
              'hf:one',
              'hf:two',
            ],
          },
        },),).toBe(undefined,);
      },
    },),
    it({
      name: 'REFUSES a role configured with nobody in it, because a stage that can never speak '
        + 'settles exactly like one whose voices all failed: the run looks clean rather than '
        + 'misconfigured, and that is the most expensive place for a failure to be silent',
      fn: async () => {
        /**
         Failure the check raised.
         */
        const caught = configurationFailure({
          roles: {
            editorModelIds: ['hf:one',],
            judgeModelIds: [],
          },
        },);
        expect(caught,).toBeInstanceOf(RosterConfigurationError,);
        expect(String(caught,),).toContain('judgeModelIds',);
        // The configured role is NOT named, so an operator reading the message
        // fixes what is broken rather than auditing what is not.
        expect(String(caught,),).not
          .toContain('editorModelIds',);
      },
    },),
    it({
      name: 'names EVERY empty role in one error rather than the first, since an operator fixing '
        + 'them one at a time pays a whole preflight per role',
      fn: async () => {
        /**
         Failure raised by a wholly unconfigured lane.
         */
        const caught = configurationFailure({
          roles: {
            panelModelIds: [],
            editorModelIds: [],
            judgeModelIds: ['hf:one',],
            checkerModelIds: [],
          },
        },);
        expect(caught,).toBeInstanceOf(RosterConfigurationError,);
        for (const role of [
          'panelModelIds',
          'editorModelIds',
          'checkerModelIds',
        ])
          expect(String(caught,),).toContain(role,);
        expect(String(caught,),).not
          .toContain('judgeModelIds',);
      },
    },),
    it({
      name: 'accepts an EMPTY role map, since a caller with nothing required to check is asking '
        + 'nothing rather than asserting everything',
      fn: async () => {
        expect(configurationFailure({ roles: {}, },),).toBe(undefined,);
      },
    },),
  ],
},);

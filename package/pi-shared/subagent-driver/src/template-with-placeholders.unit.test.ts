/**
 Filling a brief template: which placeholders a text carries, and the refusal when one is left unfilled. Nothing here writes a
 file, starts a run, or touches a machine.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  TemplatePlaceholderError,
  filledTemplate,
  placeholdersIn,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: placeholdersIn.name,
      children: [
        it({
          name: 'reads each placeholder once, in the order the text reads it, and ignores a name that is not one',
          fn: async () => {
            expect(placeholdersIn({
              template: 'a @FOCUS@ and a @COMMIT@, then @FOCUS@ again, and an @not-a-placeholder@',
            },),).toEqual(['FOCUS', 'COMMIT',],);
            expect(placeholdersIn({ template: 'no placeholder here', },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: filledTemplate.name,
      children: [
        it({
          name: 'fills every placeholder a text carries, leaving the prose around them alone',
          fn: async () => {
            expect(filledTemplate({
              template: 'Read @RULES@ first, then review @COMMIT@.',
              values: {
                RULES: '/repo/rules.md',
                COMMIT: 'abc123',
              },
              name: 'the brief',
            },),).toBe('Read /repo/rules.md first, then review abc123.',);
            expect(filledTemplate({
              template: 'nothing to fill',
              values: {},
              name: 'the brief',
            },),).toBe('nothing to fill',);
          },
        },),
        it({
          name: 'refuses a text whose placeholder no value fills, naming the placeholder and the values it was given',
          fn: async () => {
            try {
              filledTemplate({
                template: 'review @COMMIT@ of @FOCUS@',
                values: { COMMIT: 'abc123', },
                name: 'the brief',
              },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(TemplatePlaceholderError,);
              expect(String(error,),).toContain('@FOCUS@',);
              expect(String(error,),).toContain('COMMIT',);
              return;
            }
            throw new Error('a text with an unfilled placeholder was written',);
          },
        },),
        it({
          name: 'refuses a value that leaves a placeholder of its own behind, so a run never reads one as its task',
          fn: async () => {
            expect(() => filledTemplate({
              template: '@FOCUS@',
              values: { FOCUS: 'read @COMMIT@ first', },
              name: 'the brief',
            },),).toThrow(TemplatePlaceholderError,);
          },
        },),
      ],
    },),
  ],
},);

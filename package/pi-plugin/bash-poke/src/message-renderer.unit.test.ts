/**
 Tests for poke rendering and delivery in the built bash-poke artifact.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  cardLabel,
  constants,
  contentText,
  deliverPoke,
  pokeCardText,
  registerPokeRenderer,
  type PokeDetails,
} from '../dist/final/node/index.mjs';

import { createFakePi, plainTheme, } from './test-fixtures.ts';

//region Fixtures

/**
 Builds outcome details with every field present.
 
 @param exitCode - reported exit code
 
 @param cancelled - whether the user cancelled the job
 
 @returns details suitable for rendering cases
 
 @example
 ```ts
 detailsWith({ exitCode: 1, },);
 ```
 */
function detailsWith(
  {
    exitCode,
    cancelled,
  }: {
    readonly exitCode?: number;
    readonly cancelled?: boolean;
  } = {},
): PokeDetails {
  return {
    command: 'make',
    cancelled: cancelled ?? false,
    truncated: false,
    elidedChars: 0,
    outputChars: 4,
    ...(exitCode === undefined ? {} : { exitCode, }),
  };
}

/**
 Builds a poke message as Pi would hand it to a renderer.
 
 @param details - outcome data, omitted to test the absent path
 
 @param content - model-facing text
 
 @returns message payload for renderer cases
 
 @example
 ```ts
 pokeMessage({ content: 'body', },);
 ```
 */
function pokeMessage(
  {
    details,
    content = 'body',
  }: {
    readonly details?: PokeDetails;
    readonly content?: string;
  } = {},
) {
  return {
    role: 'custom' as const,
    customType: constants.POKE_CUSTOM_TYPE,
    content,
    display: true,
    timestamp: 0,
    ...(details === undefined ? {} : { details, }),
  };
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    //region contentText

    describe({
      name: contentText.name,
      children: [
        it({
          name: 'passes string content through',
          fn: async () => {
            expect(contentText({ content: 'plain', }, ), ).toBe('plain');
          },
        }, ),
        it({
          name: 'joins text parts and skips non-text parts',
          fn: async () => {
            expect(contentText({
              content: [
                { type: 'text', text: 'one', },
                { type: 'image', },
                { type: 'text', text: 'two', },
              ],
            }, ), ).toBe('one\ntwo');
          },
        }, ),
        it({
          name: 'drops empty text parts',
          fn: async () => {
            expect(contentText({
              content: [{ type: 'text', text: '', }, { type: 'text', text: 'kept', }, ],
            }, ), ).toBe('kept');
          },
        }, ),
      ],
    }, ),

    //endregion contentText

    //region cardLabel

    describe({
      name: cardLabel.name,
      children: [
        it({
          name: 'names only the package when details are absent',
          fn: async () => {
            expect(cardLabel({}, ), ).toBe('bash-poke');
          },
        }, ),
        it({
          name: 'adds the exit code when one arrived',
          fn: async () => {
            expect(cardLabel({ details: detailsWith({ exitCode: 1, }, ), }, ), )
              .toBe('bash-poke exit 1');
          },
        }, ),
        it({
          name: 'reports cancellation instead of an exit code',
          fn: async () => {
            expect(cardLabel({ details: detailsWith({ cancelled: true, }, ), }, ), )
              .toBe('bash-poke cancelled');
          },
        }, ),
        it({
          name: 'reports an unknown exit when no code arrived',
          fn: async () => {
            expect(cardLabel({ details: detailsWith(), }, ), ).toBe('bash-poke exit unknown');
          },
        }, ),
      ],
    }, ),

    //endregion cardLabel

    //region pokeCardText

    describe({
      name: pokeCardText.name,
      children: [
        it({
          name: 'puts the label above the model-facing body',
          fn: async () => {
            const text = pokeCardText({
              message: pokeMessage({ details: detailsWith({ exitCode: 2, }, ), content: 'FAILED', }, ),
              theme: plainTheme(),
            }, );
            expect(text, ).toBe('bash-poke exit 2\nFAILED');
          },
        }, ),
        it({
          name: 'renders without details',
          fn: async () => {
            const text = pokeCardText({
              message: pokeMessage({ content: 'body', }, ),
              theme: plainTheme(),
            }, );
            expect(text, ).toBe('bash-poke\nbody');
          },
        }, ),
      ],
    }, ),

    //endregion pokeCardText

    //region registerPokeRenderer

    describe({
      name: registerPokeRenderer.name,
      children: [
        it({
          name: 'registers one renderer under the poke custom type',
          fn: async () => {
            const fake = createFakePi();
            registerPokeRenderer(fake.api, );
            expect(fake.renderers.size, ).toBe(1);
            expect(fake.renderers.has(constants.POKE_CUSTOM_TYPE, ), ).toBe(true);
          },
        }, ),
        it({
          name: 'renders a component carrying the label and body',
          fn: async () => {
            const fake = createFakePi();
            registerPokeRenderer(fake.api, );

            /**
             Renderer Pi would call for a poke message.
             */
            const renderer = fake.renderers.get(constants.POKE_CUSTOM_TYPE, );
            if (renderer === undefined)
              throw new Error('expected a registered renderer');

            /**
             Component produced for one message.
             */
            const component = renderer(
              pokeMessage({ details: detailsWith({ exitCode: 0, }, ), content: 'done', }, ),
              { expanded: false, outputPad: 0, },
              plainTheme(),
            );
            if (component === undefined)
              throw new Error('expected a rendered component');
            expect(component.render(80, ).join('\n', ), ).toContain('bash-poke exit 0');
            expect(component.render(80, ).join('\n', ), ).toContain('done');
          },
        }, ),
      ],
    }, ),

    //endregion registerPokeRenderer

    //region deliverPoke

    describe({
      name: deliverPoke.name,
      children: [
        it({
          name: 'sends a custom message that triggers a follow-up turn',
          fn: async () => {
            const fake = createFakePi();
            const delivery = deliverPoke({
              pi: fake.api,
              content: '[bash finished] $ true (exit 0)',
              details: detailsWith({ exitCode: 0, }, ),
            }, );
            expect(delivery.delivered, ).toBe(true);
            expect(fake.sentMessages, ).toHaveLength(1);

            /**
             Sole recorded delivery.
             */
            const [sent] = fake.sentMessages;
            if (sent === undefined)
              throw new Error('expected one sent message');
            expect(sent.message.customType, ).toBe(constants.POKE_CUSTOM_TYPE);
            expect(sent.message.display, ).toBe(true);
            expect(sent.message.content, ).toBe('[bash finished] $ true (exit 0)');
            expect(sent.options, ).toEqual({ triggerTurn: true, deliverAs: 'followUp', });
          },
        }, ),
        it({
          name: 'reports a stale runtime instead of throwing',
          fn: async () => {
            const fake = createFakePi({
              sendMessage(): void {
                throw new Error('This extension ctx is stale after session replacement');
              },
            }, );
            const delivery = deliverPoke({
              pi: fake.api,
              content: 'text',
              details: detailsWith(),
            }, );
            expect(delivery.delivered, ).toBe(false);
            if (delivery.delivered)
              throw new Error('expected a failed delivery');
            expect(String(delivery.error, ), ).toContain('stale');
          },
        }, ),
      ],
    }, ),

    //endregion deliverPoke
  ],
}, );

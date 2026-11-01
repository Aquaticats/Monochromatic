/**
 Unit tests for the canonical Advisor tool value and result projection.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { JsonValue, } from '@deepseek-ai/dsh-util-values';
import type { AdvisorDetails, } from '@monochromatic-dev/agent-harness-shared-advisor/ts';

import {
  advisorDetailsAsJsonValue,
  advisorToolValueFromJson,
  renderAdvisorToolContent,
} from '../dist/final/node/index.mjs';

/**
 Build one minimal Advisor details fixture with an optional operation view.

 @param operation - optional persisted operation view to record

 @returns details record accepted by the renderers

 @example
 ```ts
 const details = detailsFixture();
 ```
 */
function detailsFixture(
  operation?: JsonValue,
): AdvisorDetails {
  return {
    selectedSlug: 'p/m',
    provider: 'p',
    scopeSource: 'settings',
    scopedSlugs: ['p/m',],
    ...(operation === undefined ? {} : { operation, }),
  } as unknown as AdvisorDetails;
}

await describe({
  name: advisorDetailsAsJsonValue.name,
  children: [
    it({
      name: 'round-trips details to a plain JSON value',
      fn: async () => {
        const value = advisorDetailsAsJsonValue(detailsFixture(),);
        expect(value,).toEqual(detailsFixture(),);
      },
    },),
    it({
      name: 'throws when details normalize to non-JSON data',
      fn: async () => {
        let thrown: unknown;
        try {
          advisorDetailsAsJsonValue({
            ...detailsFixture(),
            bad: (): string => 'never serializable',
          } as unknown as AdvisorDetails,);
        }
        catch (error) {
          thrown = error;
        }
        expect(Error.isError(thrown,),).toBe(true,);
        expect(
          (thrown as Error).message.includes('not JSON data',),
        ).toBe(true,);
      },
    },),
  ],
},);

await describe({
  name: advisorToolValueFromJson.name,
  children: [
    it({
      name: 'wraps a bare string as the tool text',
      fn: async () => {
        expect(advisorToolValueFromJson('hello',),).toEqual({
          text: 'hello',
          details: {},
          progress: '',
        },);
      },
    },),
    it({
      name: 'renders non-object values through JSON text',
      fn: async () => {
        expect(advisorToolValueFromJson([1, 2,],).text,).toBe('[1,2]',);
        expect(advisorToolValueFromJson(42,).text,).toBe('42',);
      },
    },),
    it({
      name: 'reads the object fields with safe fallbacks',
      fn: async () => {
        expect(
          advisorToolValueFromJson({
            text: 'body',
            details: {
              a: 1,
            },
            progress: 'tail',
          },),
        ).toEqual({
          text: 'body',
          details: {
            a: 1,
          },
          progress: 'tail',
        },);
        expect(
          advisorToolValueFromJson({
            text: 5,
            details: 'no',
            progress: 7,
          },),
        ).toEqual({
          text: '{"text":5,"details":"no","progress":7}',
          details: {},
          progress: '',
        },);
      },
    },),
  ],
},);

await describe({
  name: renderAdvisorToolContent.name,
  children: [
    it({
      name: 'leads with the operation summary and folds the progress tail',
      fn: async () => {
        const blocks = renderAdvisorToolContent({
          text: 'review text',
          details: {
            operation: {
              attempts: [
                {
                  model: 'p/m',
                  attempt: 1,
                  state: 'succeeded',
                },
              ],
              reviews: [
                {
                  model: 'p/m',
                },
              ],
              usageIncomplete: false,
            },
          },
          progress: 'finished',
        },);
        expect(blocks.length,).toBe(1,);
        const text = blocks[0]?.type === 'text' ? blocks[0].text : '';
        expect(text.endsWith('\n\nfinished',),).toBe(true,);
        expect(text.includes('p/m',),).toBe(true,);
      },
    },),
    it({
      name: 'falls back to raw text when details are not renderable',
      fn: async () => {
        const blocks = renderAdvisorToolContent({
          text: 'raw body',
          details: {
            unexpected: true,
          },
          progress: '',
        },);
        const text = blocks[0]?.type === 'text' ? blocks[0].text : '';
        expect(text,).toBe('raw body',);
      },
    },),
    it({
      name: 'renders the full summary when details are structured without an operation',
      fn: async () => {
        const blocks = renderAdvisorToolContent({
          text: 'review text',
          details: detailsFixture() as unknown as JsonValue,
          progress: 'tail',
        },);
        const text = blocks[0]?.type === 'text' ? blocks[0].text : '';
        expect(text.includes('review text',),).toBe(true,);
        expect(text.endsWith('\n\ntail',),).toBe(true,);
      },
    },),
  ],
},);

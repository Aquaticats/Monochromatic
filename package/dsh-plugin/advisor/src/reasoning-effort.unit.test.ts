/**
 Unit tests for the DSH reasoning-effort eligibility rule.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ReasoningEffortId, } from '@deepseek-ai/dsh-llm';

import { selectDshAdvisorReasoningEfforts, } from '../dist/final/node/index.mjs';

await describe({
  name: selectDshAdvisorReasoningEfforts.name,
  children: [
    it({
      name: 'keeps every effort that is not the max effort, least to greatest',
      fn: async () => {
        const efforts = selectDshAdvisorReasoningEfforts({
          efforts: [
            {
              id: ReasoningEffortId('low',),
              name: 'low',
            },
            {
              id: ReasoningEffortId('high',),
              name: 'high',
            },
            {
              id: ReasoningEffortId('max',),
              name: 'max',
            },
          ],
        },);
        expect([...efforts,],).toEqual([
          'low',
          'high',
        ],);
      },
    },),
    it({
      name: 'excludes the max effort by id even when its name differs',
      fn: async () => {
        const efforts = selectDshAdvisorReasoningEfforts({
          efforts: [
            {
              id: ReasoningEffortId('max',),
              name: 'highest',
            },
            {
              id: ReasoningEffortId('mid',),
              name: 'max',
            },
            {
              id: ReasoningEffortId('low',),
              name: 'low',
            },
          ],
        },);
        expect([...efforts,],).toEqual(['low',],);
      },
    },),
    it({
      name: 'yields an empty list when only the max effort is advertised',
      fn: async () => {
        const efforts = selectDshAdvisorReasoningEfforts({
          efforts: [
            {
              id: ReasoningEffortId('max',),
              name: 'max',
            },
          ],
        },);
        expect([...efforts,],).toEqual([],);
      },
    },),
    it({
      name: 'yields an empty list when the route advertises no reasoning',
      fn: async () => {
        expect([...selectDshAdvisorReasoningEfforts(undefined,),],).toEqual([],);
      },
    },),
  ],
},);

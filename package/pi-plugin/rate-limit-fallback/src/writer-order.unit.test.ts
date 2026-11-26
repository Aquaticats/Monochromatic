/**
 The writer order the owner set on 2026-11-26, read from the built package's own pool table.
 @module

 `doc/decision/writer-rounds-take-qwen-first-through-the-virtual-writer.md` answers the writers' order as
 "Qwen, opencode-go, flex (Recommended)". Standard NeuralWatt DeepSeek left the writer pool that day, and the
 provider named `deepseek` is never a member of either pool. The cases below fail if either comes back, so a
 later edit to `src/model-pools.ts` cannot quietly restore them.
 */

import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import {
  POOL_DEFINITIONS,
  poolMembers,
} from '../dist/final/node/index.mjs';
import { labelOf, WRITER_TARGETS, } from './member-order-fixture.ts';

/**
 Public model id of the writer pool's virtual model, whose physical model carries the members.
 */
const WRITER_POOL_ID = 'writer-pool';

/**
 Public model id of the reviewer pool's physical model.
 */
const REVIEWER_POOL_ID = 'reviewer-pool';

/**
 Provider whose DeepSeek model the owner closed to writers on 2026-11-11 and replaced on 2026-11-26.
 */
const CLOSED_PROVIDER = 'neuralwatt';

/**
 Model the closed provider's writer entry asked for.
 */
const CLOSED_MODEL = 'deepseek-v4.1-flash';

/**
 Provider no pool may ever ask, for any run.
 */
const FORBIDDEN_PROVIDER = 'deepseek';

/**
 Exact member labels the writer pool asks, in the owner's order.
 */
const WRITER_LABELS: readonly string[] = WRITER_TARGETS.map(function labelOfTarget(target,) {
  return labelOf(target,);
},);

/**
 Member labels one pool asks, as `provider/id`.

 @param modelId - physical pool model Pi asks the router for

 @returns the labels in the order the pool asks them
 */
function labelsOf(modelId: string,): readonly string[] {
  return poolMembers(modelId,)
    .map(function labelOfMember(member,) {
      return `${member.provider}/${member.id}`;
    },);
}

await describe({
  name: 'the writer pool',
  children: [
    it({
      name: 'asks the owner\'s three members in the owner\'s order',
      fn: async function ownerOrder() {
        expect(labelsOf(WRITER_POOL_ID,),).toEqual(WRITER_LABELS,);
      },
    },),
    it({
      name: 'asks no member of the closed standard NeuralWatt lane',
      fn: async function noClosedLane() {
        expect(poolMembers(WRITER_POOL_ID,)
          .filter(function isTheClosedLane(member,) {
            return (member.provider === CLOSED_PROVIDER) && (member.id === CLOSED_MODEL);
          },),).toEqual([],);
      },
    },),
    it({
      name: 'asks no member of the provider named deepseek',
      fn: async function noForbiddenProvider() {
        expect(poolMembers(WRITER_POOL_ID,)
          .filter(function isTheForbiddenProvider(member,) {
            return member.provider === FORBIDDEN_PROVIDER;
          },),).toEqual([],);
      },
    },),
    it({
      name: 'offers one virtual writer and one virtual reviewer model',
      fn: async function twoPools() {
        expect(POOL_DEFINITIONS.map(function modelIdOf(pool,) {
          return pool.modelId;
        },),).toEqual([WRITER_POOL_ID, REVIEWER_POOL_ID,],);
      },
    },),
    it({
      name: 'asks no member of the provider named deepseek in any pool',
      fn: async function noForbiddenProviderAnywhere() {
        for (const pool of POOL_DEFINITIONS) {
          expect(pool.members
            .filter(function isTheForbiddenProvider(member,) {
              return member.provider === FORBIDDEN_PROVIDER;
            },),).toEqual([],);
        }
      },
    },),
  ],
},);

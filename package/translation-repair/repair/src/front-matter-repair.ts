import type { ChunkCriticPhase, } from './chunk-critic-phase.ts';
import type { ChunkRepairOutcome, } from '@monochromatic-dev/translation-repair-assembly/ts';
import { unchangedChunkOutcome, } from './repair-unchanged-outcome.ts';

//region Front matter repair no-op
// Front matter is rendered by translate ensemble under YAML-specific rules.
// Repair lane emits an explicit stable row rather than sending syntax through
// prose-oriented critics, editors, and naturalness rewriters.

/**
 Empty critic evidence for syntax role repair does not run.
 */
const FRONT_MATTER_CRITIC: ChunkCriticPhase = {
  claims: [],
  nonTranslationVotes: 0,
  contradicted: false,
  votesStand: false,
  heardCritics: 0,
  heardCriticIds: [],
  claimAttributions: [],
  findings: [],
};

/**
 Creates explicit unchanged repair settlement for front matter.

 @param sliceIndex - global front matter slice index

 @param targetText - archive metadata retained by repair lane

 @returns Stable no-op outcome with syntax finding

 @example
 ```ts
 const outcome = frontMatterRepairOutcome({ sliceIndex: 0, targetText, });
 ```
 */
export function frontMatterRepairOutcome(
  {
    sliceIndex,
    targetText,
  }: {
    readonly sliceIndex: number;
    readonly targetText: string;
  },
): ChunkRepairOutcome {
  return {
    ...unchangedChunkOutcome({
      sliceIndex,
      targetText,
      critic: FRONT_MATTER_CRITIC,
    },),
    issues: [],
    findings: ['repair-front-matter-not-applicable (translate ensemble owns YAML metadata)',],
  };
}

//endregion Front matter repair no-op

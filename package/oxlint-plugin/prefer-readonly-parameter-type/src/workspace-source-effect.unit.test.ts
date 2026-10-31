import { readFileSync, } from 'node:fs';
import { fileURLToPath, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { isFunctionLikeDeclaration, } from 'typescript/ast/is';

import {
  buildEffectSummaryIndex,
  closeSemanticBridge,
  NO_EFFECT_SUMMARY,
  openSemanticFile,
} from '../dist/final/node/index.mjs';

/**
 Fixture source calling workspace `toml-edit-core` functions (package/test-fixture/effect-subject-toml-edit).
 */
const APPLY_PLAN_PATH = fileURLToPath(new URL(
  '../../../test-fixture/effect-subject-toml-edit/src/cargo/apply-plan.ts',
  import.meta.url,
),);

/**
 Current apply-plan source text.
 */
const SOURCE = readFileSync(
  APPLY_PLAN_PATH,
  'utf8',
);

await describe({
  name: 'workspace source effect resolution',
  concurrency: 1,
  children: [
    it({
      name: 'resolves workspace package calls through live source without catalog entries',
      fn: async () => {
        const session = openSemanticFile({
          fileName: APPLY_PLAN_PATH,
          sourceText: SOURCE,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
        },);
        /* The tomlSet chain formerly forced @mutates contracts up through
         * applyCargoPlan; live workspace analysis plus plain-data traversal
         * narrowing proves the whole chain mutation-free.
         *
         * Mutation-free, not fully resolved. Every link reaches `Object.entries` inside
         * `@monochromatic-dev/toml-edit-core`, which nothing derives, so each one
         * carries unproven reachability on the parameter it packages into that call.
         * The first two links read `[]` until the owned call edge stopped filtering
         * argument literals by the callee's authored `@mutates` names, which had let a
         * comment decide which caller-owned values inherited a callee's opacity. What
         * this case pins is the mutation column: it stays empty for all three, which is
         * the claim the catalog removal was about. */
        const summaries = [
          'setIfDiffers',
          'applyEnforcement',
          'applyCargoPlan',
        ].map(function summaryFor(functionName,) {
          const nameNode = session.nodeAtOffset(
            SOURCE.indexOf(`function ${functionName}`,) + 'function '.length,
          );
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected function declaration for ${functionName}.`,);
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected effect summary for ${functionName}.`,);
          return {
            functionName,
            mutated: [...summary.mutatedParameterIndexes,],
            opaque: [...summary.opaqueParameterIndexes,],
          };
        },);
        closeSemanticBridge();
        expect(summaries,).toEqual([
          {
            functionName: 'setIfDiffers',
            mutated: [],
            opaque: [0,],
          },
          {
            functionName: 'applyEnforcement',
            mutated: [],
            opaque: [0,],
          },
          {
            functionName: 'applyCargoPlan',
            mutated: [],
            opaque: [0,],
          },
        ],);
      },
    },),
  ],
},);

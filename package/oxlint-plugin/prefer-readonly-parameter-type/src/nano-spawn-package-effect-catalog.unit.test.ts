import { readFileSync, } from 'node:fs';
import { fileURLToPath, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  closeSemanticBridge,
  intrinsicCallableEffectQuery,
  intrinsicEffect,
  NO_INTRINSIC_EFFECT,
  NO_INTRINSIC_QUERY,
  openSemanticFile,
} from '../dist/final/node/index.mjs';
import { isIdentifier, } from 'typescript/unstable/ast/is';

/**
 * cli-git source containing three-argument nano-spawn call.
 */
const CALL_SITE_PATH = fileURLToPath(new URL(
  '../../../git-policy/cli/src/optional/forbidden-strings/scan-candidates.ts',
  import.meta.url,
),);

await describe({
  name: 'nano-spawn package effects',
  children: [
    it({
      name: 'retains only options state while observing arguments and options',
      fn: async () => {
        const sourceText = readFileSync(CALL_SITE_PATH, 'utf8',);
        const session = openSemanticFile({
          fileName: CALL_SITE_PATH,
          sourceText,
          hasBOM: false,
        },);
        const identifier = session.nodeAtOffset(sourceText.indexOf('nanoSpawn(',) + 1,);
        if (!isIdentifier(identifier,))
          throw new Error('Expected nanoSpawn identifier.',);
        const symbol = session.checker.getResolvedSymbol(identifier,);
        const query = symbol === undefined
          ? NO_INTRINSIC_QUERY
          : intrinsicCallableEffectQuery({
            project: session.project,
            memberSymbol: symbol,
          },);
        if (query === NO_INTRINSIC_QUERY)
          throw new Error('Expected nano-spawn package identity.',);
        const effect = intrinsicEffect(query,);
        closeSemanticBridge();
        if (effect === NO_INTRINSIC_EFFECT)
          throw new Error('Expected nano-spawn effect.',);
        expect(effect.targets,).toEqual([{
          kind: 'argument',
          index: 2,
        },],);
        expect(effect.opaqueTargets,).toEqual([
          {
            kind: 'argument',
            index: 1,
          },
          {
            kind: 'argument',
            index: 2,
          },
        ],);
      },
    },),
  ],
},);

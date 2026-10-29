import { caughtValueStack, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  ESTree,
  VisitorWithHooks,
} from '@oxlint/plugins';

import { verifyOverloadConsistency, } from './overload-consistency.ts';
import type { ReadonlyRuleCategory, } from './readonly-rule-category.ts';
import { readonlySourceEvidence, } from './readonly-source-evidence.ts';
import { SemanticBridgeError, } from './semantic-bridge-error.ts';
import { reportReadonlyCallableEvidence, } from './verifier.ts';

/**
 Enforced TypeScript source suffixes excluding declaration variants.
 */
const ENFORCED_SOURCE_SUFFIXES: readonly string[] = [
  '.ts',
  '.mts',
  '.cts',
  '.tsx',
];

/**
 Exempt declaration-file suffixes.
 */
const DECLARATION_SOURCE_SUFFIXES: readonly string[] = [
  '.d.ts',
  '.d.mts',
  '.d.cts',
];

/**
 Tests whether host file belongs to semantic enforcement inputs.
 
 @param fileName - Oxlint source filename.
 
 @returns whether source is non-declaration TypeScript.
 
 @example
 ```ts
 isEnforcedTypeScriptSource('source.ts');
 ```
 */
function isEnforcedTypeScriptSource(fileName: string,): boolean {
  return ENFORCED_SOURCE_SUFFIXES.some(function enforced(suffix,): boolean {
    return fileName.endsWith(suffix,);
  },)
    && (!DECLARATION_SOURCE_SUFFIXES.some(function declaration(suffix,): boolean {
      return fileName.endsWith(suffix,);
    },));
}

/**
 Creates rule-specific reporting visitor over shared source evidence.
 
 @param context - Rule context receiving only selected category.
 
 @param category - Evidence category owned by public rule.
 
 @returns visitor sharing semantic computation with sibling rules.
 
 @mutates context - Emits selected diagnostics through foreign rule context.
 
 @example
 ```ts
 createReadonlyRuleVisitor({ context, category: 'preference' });
 ```
 */
export function createReadonlyRuleVisitor({
  context,
  category,
}: ForeignBorrowed<{
  readonly context: Context;
  readonly category: ReadonlyRuleCategory;
}>,): VisitorWithHooks {
  return {
    Program(_node: ForeignBorrowed<ESTree.Program>,): void {
      if (!isEnforcedTypeScriptSource(context.filename,))
        return;
      /**
       Rule-category logger for current reporter.
       */
      const rl = tagged({ tag: category, },);
      try {
        /**
         Complete source evidence shared by every category visitor.
         */
        const evidence = readonlySourceEvidence({ context, },);
        evidence.callables
          .forEach(function reportCallable(callableEvidence,): void {
            reportReadonlyCallableEvidence({
              context,
              evidence: callableEvidence,
              category,
            },);
          },);
        if (category === 'effect-contract') {
          verifyOverloadConsistency({
            context,
            project: evidence.project,
            sourceFile: evidence.sourceFile,
            effectIndex: evidence.effectIndex,
          },);
        }
      }
      catch (error) {
        rl.warn(
          `semantic evidence unavailable for ${context.filename}: ${
            error instanceof SemanticBridgeError
              ? `${error.reason}: ${caughtValueStack(error,)}`
              : caughtValueStack(error,)
          }`,
        );
      }
    },
  };
}

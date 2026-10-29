/**
 Mutant enumeration: parse one source file and emit every mutant.
 
 Runs host-side; containers only ever see the resulting span manifest.
 
 @example
 ```ts
 enumerateMutants({ file: 'src/a.ts', source: 'const x = 1 + 2;' });
 ```
 */

import {
  type Diagnostic,
  langFromPath,
  type Node,
  parse,
} from 'yuku-parser';
import {
  is,
  walk,
} from 'yuku-ast';

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  lineStarts,
  positionAt,
} from './lines.ts';
import { mutantId, } from './mutant-id.ts';
import { isEstreeNode, } from './node-access.ts';
import { allOperators, } from './operator/index.ts';
import {
  matchingSuppressions,
  suppressionRules,
} from './suppression.ts';
import type {
  Mutant,
  Replacement,
} from './types.ts';

/**
 Mutant excluded by a suppression comment, kept for report honesty.
 */
export type IgnoredMutant = Mutant & {
  readonly reason: string;
};

/**
 Enumeration output: runnable mutants plus suppressed ones.
 */
export type EnumerationResult = {
  readonly mutants: readonly Mutant[];
  readonly ignored: readonly IgnoredMutant[];
};

/**
 Builds a full mutant from one replacement.
 
 @param options - Replacement, file identity, source, and line table.
 
 @returns Identified mutant with position info.
 
 @example
 ```ts
 toMutant({ replacement, file: 'src/a.ts', source, table });
 ```
 */
function toMutant(options: {
  readonly replacement: Replacement;
  readonly file: string;
  readonly source: string;
  readonly table: readonly number[];
},): Mutant {
  /**
   One-based line and zero-based column of the replacement start.
   */
  const position = positionAt({
    table: options.table,
    offset: options.replacement
      .start,
  },);

  return {
    id: mutantId({
      file: options.file,
      start: options.replacement
        .start,
      end: options.replacement
        .end,
      operator: options.replacement
        .operator,
      replacement: options.replacement
        .text,
    },),
    file: options.file,
    start: options.replacement
      .start,
    end: options.replacement
      .end,
    line: position.line,
    column: position.column,
    operator: options.replacement
      .operator,
    original: options.source
      .slice(
      options.replacement
        .start,
      options.replacement
        .end,
      ),
    replacement: options.replacement
      .text,
    description: options.replacement
      .description,
  };
}

/**
 Returns whether a node roots a TypeScript type-only subtree.
 
 The compiler erases these subtrees, so mutants inside them can never
 change runtime behaviour; skipping them at walk level saves one
 per-mutant compile check each. Runtime-emitting TypeScript constructs
 (enums, namespaces, parameter properties) stay walkable.
 
 @param node - Candidate node from the walk.
 
 @returns Whether the subtree is compile-time-only.
 
 @example
 ```ts
 isTypeErasedRoot(typeAliasDeclarationNode);
 // true
 ```
 */
function isTypeErasedRoot(node: Readonly<Node>,): boolean {
  return is.TSType(node,)
    || is.TSTypeAnnotation(node,)
    || is.TSTypeAliasDeclaration(node,)
    || is.TSInterfaceDeclaration(node,)
    || is.TSDeclareFunction(node,)
    || is.TSTypeParameterDeclaration(node,)
    || is.TSTypeParameterInstantiation(node,);
}

/**
 Enumerates every mutant in one source file.
 
 Identical (span, replacement) pairs from overlapping families dedupe;
 output is sorted by span then replacement so manifests stay stable
 across runs.
 
 @param options - Package-relative file path and its source text.
 
 @returns Runnable mutants plus suppression-ignored mutants.
 
 @throws Error when the file does not parse.
 
 @example
 ```ts
 const { mutants, ignored } = enumerateMutants({ file: 'src/a.ts', source });
 ```
 */
export function enumerateMutants(options: {
  readonly file: string;
  readonly source: string;
},): EnumerationResult {
  /**
   Parse result carrying program, comments, and diagnostics; yuku-parser
   recovers from syntax errors, so severity filtering decides rejection.
   */
  const parsed = parse(
    options.source,
    { lang: langFromPath(options.file,), },
  );

  /**
   Error-severity diagnostics; warnings and hints stay non-fatal.
   */
  const parseErrors = parsed.diagnostics
    .filter(function isError(diagnostic: ForeignBorrowed<Diagnostic>,): boolean {
      return diagnostic.severity === 'error';
    },);

  if (parseErrors.length > 0)
    throw new Error(
      `parse of ${options.file} failed: ${parseErrors
        .map(function toMessage(diagnostic: ForeignBorrowed<Diagnostic>,): string {
          return diagnostic.message;
        },)
        .join('; ',)}`,
    );

  /**
   Line-start offsets shared by position math and suppressions.
   */
  const table = lineStarts(options.source,);
  /**
   Parsed suppression rules from the file's comments.
   */
  const rules = suppressionRules({
    comments: parsed.comments,
    table,
  },);
  /**
   All replacements collected across operators, pre-dedupe.
   */
  const collected: Replacement[] = [];

  walk(
    parsed.program,
    {
      enter: function visitNode(
        node: Readonly<Node>,
        ctx,
      ): void {
        if (isTypeErasedRoot(node,)) {
          ctx.skip();
          return;
        }

        if (!isEstreeNode(node,))
          return;

        /**
         Structural parent view for operators, absent at the walk root.
         */
        const { parent, } = ctx;

        for (const operator of allOperators) {
          collected.push(...operator({
            node,
            ...(((parent !== null) && isEstreeNode(parent,)) ? { parent, } : {}),
            source: options.source,
          },),);
        }
      },
    },
  );

  /**
   Replacements deduped by span plus replacement text.
   */
  const deduped = [
    ...new Map(collected.map(function bySpanText(replacement,): readonly [
      string,
      Replacement,
    ] {
      return [
        `${String(replacement.start,)}:${String(replacement.end,)}:${replacement.text}`,
        replacement,
      ];
    },),)
      .values(),
  ]
    .toSorted(function bySpan(
      a,
      b,
    ): number {
      return (a.start - b.start)
        || (a.end - b.end)
        || (a.text < b.text ? -1 : 1);
    },);

  /**
   Runnable mutants accumulated during suppression filtering.
   */
  const mutants: Mutant[] = [];
  /**
   Suppressed mutants accumulated during suppression filtering.
   */
  const ignored: IgnoredMutant[] = [];

  for (const replacement of deduped) {
    /**
     Full mutant with identity and position.
     */
    const mutant = toMutant({
      replacement,
      file: options.file,
      source: options.source,
      table,
    },);
    /**
     Suppression rules covering this mutant, empty when runnable.
     */
    const matches = matchingSuppressions({
      rules,
      line: mutant.line,
      operator: mutant.operator,
    },);
    /**
     Highest-priority covering rule, when any.
     */
    const [match,] = matches;

    if (match === undefined)
      mutants.push(mutant,);
    else
      ignored.push({
        ...mutant,
        reason: match.reason,
      },);
  }

  return {
    mutants,
    ignored,
  };
}

import type { ParameterDeclaration, } from 'typescript/ast';
import { isIdentifier, } from 'typescript/ast/is';

import type { ParameterIndex, } from './effect-slot-identity.ts';

/**
 Joins quoted local binding names for diagnostic prose.
 
 @param names - Source-ordered local binding names.
 
 @returns human-readable quoted binding list.
 
 @example
 ```ts
 quotedBindings(['raw', 'rest']);
 ```
 */
function quotedBindings(names: readonly string[],): string {
  /**
   Searchable local identifiers with diagnostic delimiters.
   */
  const quoted = names.map(function quoteBinding(name,): string {
    return `"${name}"`;
  },);
  if (quoted.length === 1)
    return quoted[0] ?? '"unknown"';
  if (quoted.length === 2)
    return `${quoted[0]} and ${quoted[1]}`;
  /**
   Final binding joined after comma-separated leading bindings.
   */
  const finalBinding = quoted.at(-1,) ?? '"unknown"';
  /**
   Leading bindings preserving declaration order.
   */
  const leadingBindings = quoted
    .slice(
    0,
    -1,
    )
    .join(', ',);
  return `${leadingBindings}, and ${finalBinding}`;
}

/**
 Builds stable one-line subject for one declared parameter.
 
 Identifier parameters retain their authored name.
 Binding patterns name searchable local bindings rather than raw source,
 whose whitespace,
 comments,
 defaults,
 and nesting can span several output lines.
 Empty patterns fall back to declared position because they bind no searchable name.
 
 @param parameter - Parameter whose diagnostic subject is needed.
 
 @param parameterIndex - Semantic parameter position.
 
 @param targetIndexes - Local binding names mapped to parameter positions.
 
 @returns one-line parameter subject suitable for every public rule.
 
 @example
 ```ts
 parameterDiagnosticSubject({ parameter, parameterIndex, targetIndexes });
 ```
 */
export function parameterDiagnosticSubject({
  parameter,
  parameterIndex,
  targetIndexes,
}: {
  readonly parameter: ParameterDeclaration;
  readonly parameterIndex: ParameterIndex;
  readonly targetIndexes: ReadonlyMap<string, number>;
},): string {
  if (isIdentifier(parameter.name,)) {
    /**
     Authored identifier name with no source trivia.
     */
    const identifierName = parameter.name
      .text;
    return `Parameter "${identifierName}"`;
  }
  /**
   Local names introduced by current binding pattern in authored order.
   */
  const names: string[] = [];
  targetIndexes.forEach(function collectBinding(
    index,
    name,
  ): void {
    if (index === parameterIndex)
      names.push(name,);
  },);
  if (names.length === 0)
    return `Parameter ${String(parameterIndex + 1,)} at this location`;
  return `Destructured parameter with ${names.length === 1 ? 'binding' : 'bindings'} ${quotedBindings(names,)}`;
}

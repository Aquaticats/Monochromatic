/**
 Whether every caller of a callable is one this program can enumerate at all.
 
 @module
 */

import type { Node, } from 'typescript/ast';
import {
  type Project,
  SymbolFlags,
} from 'typescript/sync';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Caller-enumeration logger.
 */
const l = tagged({ tag: 'effect-caller-enumeration', },);

/**
 Tests whether a callable can be reached from outside the enumerated program.
 
 The completeness question `getSignatureUsage` does not answer. That query reports every
 reference TypeScript can find, and both consumers of it used to read "every reference found
 resolves" as "every caller is accounted for". Those differ exactly when references exist
 that the query cannot find, and a module export is how that happens:
 
 ```ts
 export function copyRows(rows: Row[],): Row[] {
   return rows.slice();
 }
 
 void copyRows([],);
 ```
 
 The in-program call makes the enumeration non-empty and resolvable, so requiring a caller
 does not close it. A consumer outside this program then writes `copyRows(rows,)[0].value = 1`
 with nothing attributing the write.
 
 Module export is a deliberate over-approximation of "reachable from outside". A callable
 exported only to another file inside this same program is refused too, though its callers
 are in fact all enumerable. Refusing it costs precision; admitting the published case costs
 soundness, and only one of those directions produces a wrong read-only offer.
 
 A source file with no module symbol is a script rather than a module, and nothing can import
 from it, so its callables are enumerable.
 
 @param project - TypeScript project resolving module exports.
 
 @param declaration - Callable whose reachability is in question.
 
 @returns whether every caller of this callable is one this program can enumerate.
 
 @example
 ```ts
 callersAreEnumerable({ project, declaration });
 ```
 */
export function callersAreEnumerable({
  project,
  declaration,
}: {
  readonly project: Project;
  readonly declaration: Node;
},): boolean {
  try {
    /**
     Source file whose module surface decides external reachability.
     */
    const sourceFile = declaration.getSourceFile();
    /**
     Module symbol carrying whatever this file lets other files import.
     */
    const moduleSymbol = project.checker
      .getSymbolAtLocation(sourceFile,);
    if (moduleSymbol === undefined)
      return true;
    /**
     Span identity of the callable in question.
     
     Compared by span rather than by node, because `resolve` hands back a fresh handle and
     `===` between two of them is false even for one declaration. `callableKey` settled the
     same question for the ownership graph the same way, and a first attempt here used
     identity and silently answered "not exported" for every callable, which is the
     permissive direction and would have made this predicate inert.
     */
    const target = `${sourceFile.fileName}:${String(declaration.pos,)}:${String(declaration.end,)}`;
    return !project.checker
      .getExportsOfModule(moduleSymbol,)
      .some(function exportsThisCallable(exported,): boolean {
        /**
         Export followed through any alias to what it actually names.
         */
        const resolved = (exported.flags & SymbolFlags.Alias) !== 0
          ? project.checker
            .getAliasedSymbol(exported,)
          : exported;
        return resolved.declarations
          .some(function namesThisCallable(candidate,): boolean {
            /**
             Declaration this export names, resolved into the project being walked.
             */
            const node = candidate.resolve(project,);
            if (node === undefined)
              return false;
            return `${node.getSourceFile()
              .fileName}:${String(node.pos,)}:${String(node.end,)}` === target;
          },);
      },);
  }
  catch (error) {
    /* Failing closed, and the only correct direction for both consumers. A module surface
     * this cannot read is one whose external reachability is unknown, and unknown
     * reachability has to count as reachable. */
    l.error(`module exports unavailable, treating callers as unenumerable: ${String(error,)}`,);
    return false;
  }
}

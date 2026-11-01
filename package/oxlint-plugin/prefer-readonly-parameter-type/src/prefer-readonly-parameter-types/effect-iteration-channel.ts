/**
 Whether draining an iterated value can run user code.
 
 @module
 */

import type { Node, } from 'typescript/ast';
import type { Project, } from 'typescript/sync';

/**
 Prefix the checker gives a well-known symbol member's escaped name.
 
 The suffix is a per-program identity number, `__@iterator@214` in one measured build, so the
 whole name cannot be written down and a lookup by exact key finds nothing. That is not a
 detail: an earlier attempt asked `getPropertyOfType(type, '__@iterator')`, matched nothing for
 any receiver including a plain array, and its unresolved branch then charged every iteration
 in the repository, which read as the check working.
 */
const ITERATOR_MEMBER_PREFIX = '__@iterator';

/**
 Prefix for the asynchronous iterator member, named the same way.
 */
const ASYNC_ITERATOR_MEMBER_PREFIX = '__@asyncIterator';

/**
 Tests whether draining this value reaches an iterator declared outside the default library.
 
 Draining is a call. The accepted trust baseline covers the standard iterator for a value typed
 as a collection view, recorded in `doc/decision/prefer-readonly-member-channel-authority.md`,
 and nothing checked that a given iterator was the standard one. A class whose own
 `[Symbol.iterator]` writes the object it belongs to therefore ran, mutated caller-owned state,
 and left the parameter offerable.
 
 Asked of the member rather than of the receiver's type, because those disagree exactly where it
 matters: a class implementing `Iterable` declares its own, and a class extending `Array`
 inherits the library's, so whose file declares the member answers for both without enumerating
 either.
 
 Answers "no" wherever the member cannot be found or resolved. That is deliberate: charging an
 unresolved iterator instead moves five pinned cases, including a primitive iteration target and
 a spread whose fallback type resolves to nothing.
 
 @param project - TypeScript project resolving types and declaration files.
 
 @param node - Expression a `for...of` or spread drains.
 
 @returns whether draining may reach code outside the default library.
 
 @example
 ```ts
 iterationOpensUserCode({ project, node: statement.expression });
 ```
 */
export function iterationOpensUserCode({
  project,
  node,
}: {
  readonly project: Project;
  readonly node: Node;
},): boolean {
  /**
   Type of the value being drained.
   */
  const iterated = project.checker
    .getTypeAtLocation(node,);
  if (iterated === undefined)
    return false;
  /**
   Iterator member the type exposes, found by prefix because the name carries an identity.
   */
  const member = project.checker
    .getPropertiesOfType(iterated,)
    .find(function isIteratorMember(property,): boolean {
      /**
       Escaped name the checker gave this member.
       */
      const { name, } = property;
      return name.startsWith(ITERATOR_MEMBER_PREFIX,)
        || name.startsWith(ASYNC_ITERATOR_MEMBER_PREFIX,);
    },);
  /**
   Declarations backing that member.
   */
  const declarations = member?.declarations;
  if ((declarations === undefined) || (declarations.length === 0))
    return false;
  return [...declarations,]
    .some(function declaredOutsideDefaultLibrary(handle,): boolean {
      /**
       Declaration node behind this handle, absent when it cannot be resolved.
       */
      const declaration = handle.resolve(project,);
      return (declaration !== undefined)
        && (!project.program
          .isSourceFileDefaultLibrary(declaration.getSourceFile(),));
    },);
}

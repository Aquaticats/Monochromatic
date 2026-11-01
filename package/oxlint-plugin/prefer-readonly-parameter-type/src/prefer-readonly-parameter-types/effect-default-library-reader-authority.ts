/**
 Default-library functions that read a caller-owned value passed as an argument.
 
 The collection authority answers `receiver.member(...)`, where the caller-owned value is
 the receiver. `Object.entries(value)` puts it in argument position instead, with a
 global as the receiver, so nothing in that path reaches it and every caller-owned value
 handed to one of these took an opaque boundary. Measured across the repository: 73
 findings name one of these and nothing else, so they clear entirely, and 25 more name
 one alongside another unresolved call.
 
 The channel claim lives here rather than in `effect-member-channel-authority.ts`, and
 the attempt to put it there is worth recording. That module's table is keyed by
 interface owner and consumed by a probe that calls each listed member on a receiver of
 that interface. These are static functions taking an operand, so the probe reported
 `ObjectConstructor.hasOwn is not callable on the probe receiver` and refused the
 entries. It was right to: an instance member reached through a receiver and a static
 function reached through an argument are different claims, and merging them would have
 made that table's evidence mean two things at once.
 
 The claim itself: each of these performs `Get(value, key)` for own enumerable string
 keys, which runs an accessor if the caller installed one. That is the channel
 `value.key` already opens, and this rule treats a plain property read as a pure read, so
 admitting these widens nothing that was not already assumed. Narrower than it may read,
 too: the key list is taken before any value is read, so an accessor firing mid-walk
 cannot add or remove entries from the result. `effect-default-library-reader.unit.test.ts`
 establishes both against a fully trapped object rather than restating them.
 
 @module
 */

import type {
  CallExpression,
  Expression,
  Node,
} from 'typescript/ast';
import {
  isGetAccessorDeclaration,
  isIdentifier,
  isInterfaceDeclaration,
  isMethodDeclaration,
  isMethodSignatureDeclaration,
  isSetAccessorDeclaration,
} from 'typescript/ast/is';
import type {
  Checker,
  Project,
  Type,
} from 'typescript/sync';

/**
 Reader whose result is built from the operand's own values, so it carries them.
 
 `Object.entries` and `Object.values` hand back the operand's property values inside a
 freshly allocated array. The array is fresh; the values in it are not, so a write
 through one reaches the operand and has to be attributed to it.
 */
export const READER_RESULT_CARRIES_OPERAND: unique symbol = Symbol(
  'reader result contains values held by its operand',
);

/**
 Reader whose result shares no identity with the operand.
 
 `Object.keys` returns freshly built strings and `Object.hasOwn` a boolean. Neither can
 carry a caller-owned object, so no use of the result can reach the operand.
 */
export const READER_RESULT_FRESH: unique symbol = Symbol(
  'reader result shares no identity with its operand',
);

/**
 Call that this module has not established as a verified reader.
 
 Absence is never a claim that a function mutates or dispatches, only that nothing here
 has shown it does not.
 */
export const NOT_A_VERIFIED_READER: unique symbol = Symbol(
  'call is not a verified default-library reader',
);

/**
 What a verified reader's result shares with the value it read.
 */
export type ReaderResultRelation =
  | typeof READER_RESULT_CARRIES_OPERAND
    | typeof READER_RESULT_FRESH;

/**
 Verified readers by default-library interface owner and member name.
 
 Every entry reads its first argument without writing it, and none is given a
 caller-supplied callback, so the only user code any of them can reach is an accessor or a
 proxy trap on the operand.
 
 `getPrototypeOf` is the narrowest of them: the others walk own enumerable string-keyed
 properties, while it reads one internal slot and can reach only the corresponding trap.
 It is recorded as carrying its operand rather than as fresh, which is the conservative
 reading: the prototype it hands back is reachable from the operand, and writing through
 it changes what the operand does. Measured as the largest single root cause on the
 argument side, named by 53 findings through `isPlainObject` in
 `package/toml-edit/core/src/value-encoders.ts`.
 */
const VERIFIED_READERS: Readonly<Record<string, Readonly<Record<string, ReaderResultRelation>>>> = {
  ObjectConstructor: {
    entries: READER_RESULT_CARRIES_OPERAND,
    values: READER_RESULT_CARRIES_OPERAND,
    getPrototypeOf: READER_RESULT_CARRIES_OPERAND,
    keys: READER_RESULT_FRESH,
    hasOwn: READER_RESULT_FRESH,
  },
};

/**
 Count of verified reader entries, pinned so the architecture guard can compare it.
 */
export const VERIFIED_READER_COUNT = 5;

/**
 The caller-owned value a verified reader reads, with what its result shares.
 */
export type VerifiedReaderCall = {
  readonly operand: Expression;
  readonly resultRelation: ReaderResultRelation;
};

/**
 Resolves one call to a verified default-library reader and its operand.
 
 @param project - TypeScript project proving default-library ownership.
 
 @param call - Call expression under inspection.
 
 @param declaration - Declaration the call resolved to.
 
 @returns operand and result relation, or absence when unproven.
 
 @example
 ```ts
 verifiedReaderCall({ project, call, declaration });
 ```
 */
export function verifiedReaderCall({
  project,
  checker,
  call,
  declaration,
}: {
  readonly project: Project;
  readonly checker: Checker;
  readonly call: CallExpression;
  readonly declaration: Node;
},): VerifiedReaderCall | typeof NOT_A_VERIFIED_READER {
  if ((!isMethodSignatureDeclaration(declaration,))
    || (!isIdentifier(declaration.name,))
    || (!project
      .program
      .isSourceFileDefaultLibrary(declaration.getSourceFile(),)))
    return NOT_A_VERIFIED_READER;
  /**
   Default-library interface declaring this member.
   */
  const owner = declaration.parent;
  if ((!isInterfaceDeclaration(owner,)) || (!isIdentifier(owner.name,)))
    return NOT_A_VERIFIED_READER;
  /**
   Result relation recorded for this owner and member, when one is.
   */
  const resultRelation = VERIFIED_READERS[owner.name
    .text]?.[declaration.name
      .text];
  if (resultRelation === undefined)
    return NOT_A_VERIFIED_READER;
  /**
   Value the reader reads, which is always its first argument.
   */
  const [operand,] = call.arguments;
  if (operand === undefined)
    /* Called with nothing to read. Nothing caller-owned reaches it, and there is no
     * operand to answer for, so it stays unproven rather than being answered vacuously. */
    return NOT_A_VERIFIED_READER;
  if (!operandHoldsOnlyData({
    project,
    checker,
    operand,
  },))
    /* The operand may carry something other than data properties, so enumerating it can
     * reach code this rule has not seen. `enumerateReadonlyMapEntries` in
     * `readonly-plain-data-invalid.ts` is the measured case: a `ReadonlyMap` parameter is
     * an interface whose properties are methods, and the audit that removed the
     * plain-data catalog left such an operand deliberately fail-closed. This is a
     * structural test rather than a list of admitted types, so it adds no catalog. */
    return NOT_A_VERIFIED_READER;
  return {
    operand,
    resultRelation,
  };
}

/**
 Tests whether every property an operand exposes is a plain data property.
 
 @param project - TypeScript project resolving declaration handles.
 
 @param checker - TypeScript checker resolving the operand's apparent type.
 
 @param operand - Expression the reader would enumerate.
 
 @returns whether enumeration can reach nothing but data.
 
 @example
 ```ts
 operandHoldsOnlyData({ checker, operand });
 ```
 */
function operandHoldsOnlyData({
  project,
  checker,
  operand,
}: {
  readonly project: Project;
  readonly checker: Checker;
  readonly operand: Expression;
},): boolean {
  /**
   Type the operand carries at this call.
   */
  const operandType = checker.getTypeAtLocation(operand,);
  if (operandType === undefined)
    return false;
  if (checker.getSignaturesOfType(
    operandType,
    0,
  )
    .length
    > 0)
    /* Callable, so its own properties are whatever a function object carries. */
    return false;
  return checker.getPropertiesOfType(operandType,)
    .every(function isDataProperty(property,): boolean {
      /**
       Declarations introducing this property, empty for a synthesized one.
       */
      const declarations = property.declarations ?? [];
      if (declarations.length === 0)
        /* Nothing to inspect, so nothing establishes it is data. */
        return false;
      return declarations.every(function declaresData(handle,): boolean {
        /**
         Declaration node behind this handle, absent when it cannot be resolved.
         */
        const declared = handle.resolve(project,);
        if (declared === undefined)
          return false;
        return (!isGetAccessorDeclaration(declared,))
          && (!isSetAccessorDeclaration(declared,))
          && (!isMethodDeclaration(declared,))
          && (!isMethodSignatureDeclaration(declared,));
      },);
    },);
}

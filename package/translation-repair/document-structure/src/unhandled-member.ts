import { isJsonRecord, } from './json-guard.ts';

//region Unhandled member refusal
// What a branch chain over a union ends in, so a member the chain does not
// name is a build error and, if the build is silenced, a loud failure rather
// than whatever the last branch happened to answer.
//
// A CLASS NAMED FOR A BROKEN INVARIANT, thrown where the chain ends, and not a
// function the chain returns. The coverage census counts a stretch apart from
// the cold code only where every statement in it is a throw of an invariant
// (`corpus-run/coverage-invariant-throw.ts`): a call to a helper that throws
// read as code no test ran, at every chain, for ever, which is a standing
// reason to delete the tail.

/**
 How a member no branch handled is named in the failure: by its discriminant
 when it carries one, by its runtime type otherwise.

 NAMES THE SHAPE AND NOT THE VALUE. The members this guards can carry text
 fields, and the message reaches a pass's stdout, so printing the member
 would print corpus wording on the day a union grows.

 @param member - value no branch handled

 @returns `kind <discriminant>` or `type <runtime type>`

 @example
 ```ts
 shapeOf({ member: { kind: 'purring', wording: 'Mittens naps.', }, },); // 'kind purring'
 ```
 */
function shapeOf({ member, }: { readonly member: unknown; },): string {
  return (isJsonRecord(member,) && ((typeof member.kind) === 'string'))
    ? `kind ${member.kind}`
    : `type ${typeof member}`;
}

/**
 The failure for a union member that no branch of a chain handled.

 Unreachable while the chain names every member, which the `never` parameter
 guarantees: a union that gains a member leaves the throw site passing a
 value that is not `never`, and the file stops compiling. This is built only
 if someone widens a union and silences that error with an assertion.

 Not marked safe to forward: its discriminant is read off a value the types
 say cannot exist, so a boundary prints the class alone.

 @example
 ```ts
 throw new UnhandledMemberInvariantError({ what: 'lane comparison fault', member: fault, },);
 ```
 */
export class UnhandledMemberInvariantError extends Error {
  /**
   @param what - which union was being read, for the message

   @param member - value no branch handled, typed `never` so a widened union
   fails to compile at the throw site rather than throwing here
   */
  public constructor(
    {
      what,
      member,
    }: {
      readonly what: string;
      readonly member: never;
    },
  ) {
    super(`unreachable: ${what} carries a member no branch handles: ${shapeOf({ member, },)}`,);
    this.name = 'UnhandledMemberInvariantError';
  }
}

//endregion Unhandled member refusal

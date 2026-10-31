//region Re-ask role at a call
// WHICH ROLE A CALL NAMES, READ FROM THE SOURCE TEXT (ledger B1079). The re-ask of a reasoning runaway leaves out the
// models its card holds out of the seat's role, so every call of a function that asks a bench names the role of that
// bench. A caller that cannot be driven to its gather with a scripted client in one case is held to its role by what
// its source names: this reads the argument object of each real call and returns the value of its `role`.
//
// A REAL CALL is `await callee(` or `await callee<Type>(` with the argument object opening at the end of its line, and
// the repository's formatter puts the properties of that object two columns in from the line that opens the call and
// its closing brace on a line of its own at the indentation of the opening line. So the call's own `role` is the line
// that begins `role` at exactly that depth; a `role` of a nested message sits deeper, and a one-line example in a TSDoc
// comment is no real call.

/**
 Reads the value a call's own `role` property names, from the text that follows the name of the function.

 @param call - text after `await callee`, up to the end of the file

 @param indent - indentation of the line that opens the call

 @returns One entry for a real call: the role literal, the name of the variable passed (`role` for the shorthand), or
 `none`; no entry for a text that is no real call

 @example
 ```ts
 const roles = roleOfCall({ call: '({\n  role: \'checker\',\n},);', indent: '', },);
 ```
 */
function roleOfCall(
  {
    call,
    indent,
  }: {
    readonly call: string;
    readonly indent: string;
  },
): readonly string[] {
  /**
   The text with a type argument list, when the call has one, taken off its front.
   */
  const afterTypes = call.startsWith('<',) ? call.slice(call.indexOf('>',) + 1,) : call;
  if (!afterTypes.startsWith('({\n',))
    return [];

  /**
   The argument object, from its first line to the line before its closing brace.
   */
  const body = afterTypes.slice(
    '({\n'.length,
    afterTypes.indexOf(`\n${indent}}`,),
  );

  /**
   Text a line of the object's own `role` property begins with.
   */
  const key = `${indent}  role`;

  /**
   Lines at the depth of the object's own properties that begin with `role` and then end the key.
   */
  const roleLines = body
    .split('\n',)
    .filter(function isOwnRole(line,): boolean {
      return line.startsWith(key,) && (line.startsWith(`${key}:`,) || line.startsWith(`${key},`,));
    },);

  /**
   The first such line, when the call names a role.
   */
  const [roleLine,] = roleLines;
  if (roleLine === undefined)
    return ['none',];

  /**
   What follows the key, with the key's own colon or comma still on its front.
   */
  const afterKey = roleLine.slice(key.length,);
  if (afterKey.startsWith(',',))
    return ['role',];

  /**
   The value the key names, with its trailing comma.
   */
  const named = afterKey
    .slice(1,)
    .trim();

  /**
   The value without its trailing comma.
   */
  const bare = named.endsWith(',',) ? named.slice(
    0,
    -1,
  ) : named;
  return [bare.replaceAll(
    '\'',
    '',
  ),];
}

/**
 Reads the role each real call of a function names.

 @param source - text of the file that calls the function

 @param callee - function called

 @returns One entry per real call, in source order: the role literal the call names, the name of the variable it passes,
 or `none` where it names none

 @example
 ```ts
 expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual(['wide-judge',],);
 ```
 */
export function rolesNamedAtCalls(
  {
    source,
    callee,
  }: {
    readonly source: string;
    readonly callee: string;
  },
): readonly string[] {
  /**
   The text before the first call, then the text after each `await callee`.
   */
  const pieces = source.split(`await ${callee}`,);
  return pieces
    .slice(1,)
    .flatMap(function rolesOf(
      piece,
      index,
    ): readonly string[] {
      /**
       The text before this call, whose last line holds the call's indentation.
       */
      const before = pieces[index] ?? '';

      /**
       The part of that last line in front of `await`.
       */
      const lead = before.slice(before.lastIndexOf('\n',) + 1,);

      /**
       The indentation of the line that opens the call.
       */
      const indent = lead.slice(
        0,
        lead.length
          - lead
            .trimStart()
            .length,
      );
      return roleOfCall({
        call: piece,
        indent,
      },);
    },);
}

//endregion Re-ask role at a call

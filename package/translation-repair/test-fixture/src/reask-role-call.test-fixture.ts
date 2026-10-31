import {
  afterBracket,
  propertiesOf,
  type Token,
  tokensOf,
} from './reask-role-tokens.test-fixture.ts';

//region Re-ask role at a call
// WHICH ROLE A CALL NAMES, READ FROM THE SOURCE TEXT (ledger B1079, B1100). The re-ask of a reasoning runaway leaves out
// the models its role may not take, so every call of a function that asks a bench names the role of that bench. A
// caller that cannot be driven to its gather with a scripted client in one case is held to its role by what its source
// names: this reads the argument object of each real call and returns the value of its role key.
//
// IT READS TOKENS AND NOT LINES. The source is cut into words, strings and punctuation with comments dropped, so the
// formatter's layout decides nothing: a call with `await`, a returned call, a call kept in a variable and a call on one
// line are all read, an example in a comment is none, and a declaration (`function callee`) or a method of another
// object (`.callee(`) is no call. The argument object is split into its properties at the commas that are not inside a
// bracket, so the role of a message in a prompt, which sits inside brackets, is no property of the call.

/**
 How many tokens after a variable's name the equals sign that sets it may stand, past a type annotation.
 */
const SETTING_REACH = 12;

/**
 The words that declare a variable, in front of which a name is set and not only used.
 */
const DECLARING_WORDS: ReadonlySet<string> = new Set([
  'const',
  'let',
  'var',
]);

/**
 The three dots that open a spread property.
 */
const SPREAD_MARK = '...';

/**
 Tokens that open a property holding an object literal: the key, the colon and the brace.
 */
const HOLDER_HEAD = [
  'key',
  ':',
  '{',
];

/**
 Follows a dotted key into the objects a call passes, down to the properties of the last one.

 @param properties - properties of the call's argument object

 @param path - segments before the last one in the dotted key

 @returns Properties of the object the path leads to; none where a segment names no object literal

 @example
 ```ts
 const inner = propertiesAtPath({ properties, path: ['judging',], },);
 ```
 */
function propertiesAtPath(
  {
    properties,
    path,
  }: {
    readonly properties: readonly (readonly Token[])[];
    readonly path: readonly string[];
  },
): readonly (readonly Token[])[] {
  return path.reduce(
    function descend(
      level,
      segment,
    ): readonly (readonly Token[])[] {
    /**
     The property that holds the next object.
     */
      const holder = level.find(function isSegment(property,): boolean {
        return property[0]
          ?.text
          === segment;
      },);
      if ((holder?.[1]
        ?.text
        !== ':') || (holder[2]
          ?.text
          !== '{'))
        return [];
      return propertiesOf({
        body: holder.slice(
          HOLDER_HEAD.length,
          -1,
        ),
      },);
    },
    properties,
  );
}

/**
 Whether a property is a spread, `...x`.

 @param property - tokens of one property

 @returns True when it opens with three dots
 */
function isSpread(property: readonly Token[],): boolean {
  return property
    .slice(
    0,
    SPREAD_MARK.length,
    )
    .map(function textOf(token,): string {
      return token.text;
    },)
    .join('',)
    === SPREAD_MARK;
}

/**
 Reads what one call's argument names under a key.

 @param properties - properties of the call's argument object

 @param key - key whose values are read

 @returns Value named, `none` where the key is absent, `<value> then a spread` where a spread follows the key,
 and `duplicate <key>: a, b` where the key is written twice

 @example
 ```ts
 const named = namedUnderKey({ properties, key: 'role', },);
 ```
 */
function namedUnderKey(
  {
    properties,
    key,
  }: {
    readonly properties: readonly (readonly Token[])[];
    readonly key: string;
  },
): string {
  /**
   Whether a property is written under the key.

   @param property - tokens of one property

   @returns True when its first token is the key
   */
  function isKey(property: readonly Token[],): boolean {
    return (property[0]
      ?.text
      === key)
      || isComputedKey(property,);
  }

  /**
   Whether a property is written under the key as a computed name, `['role']: ...`.

   @param property - tokens of one property

   @returns True when it opens with a bracket, the key as a string and the closing bracket
   */
  function isComputedKey(property: readonly Token[],): boolean {
    return (property[0]
      ?.text
      === '[')
      && (property[1]
        ?.kind
        === 'string')
      && (property[1]
        .text
        === key)
      && (property[2]
        ?.text
        === ']');
  }

  /**
   Values the key is given, in source order.
   */
  const values = properties
    .filter(isKey,)
    .map(function valueOf(property,): string {
      /**
       The property with a computed name read as a plain one, its three tokens of brackets and string standing as the key.
       */
      const plain = isComputedKey(property,) ? property.slice(2,) : property;
      return (plain[1]
        ?.text
        === ':')
        ? plain
          .slice(2,)
          .map(function textOf(token,): string {
            return token.text;
          },)
          .join('',)
        : key;
    },);

  /**
   Whether a property is a spread that can replace the value: one of an expression that is no inline object (a variable
   or a call, which can carry anything), or of an inline object that names the key itself. A spread of an inline object
   that names other keys, as the conditional spreads of optional arguments do, cannot.

   @param property - tokens of one property

   @returns True when it is such a spread
   */
  function mayReplaceValue(property: readonly Token[],): boolean {
    if (!isSpread(property,))
      return false;
    /**
     What the spread spreads, after its three dots.
     */
    const after = property.slice(SPREAD_MARK.length,);
    /**
     Whether it is an object written in place (`...{ a }`) or a conditional of such (`...(c ? { a } : {})`), whose keys can
     be read; any other expression (a variable, a call, even a call with an object argument) can carry anything.
     */
    const inlineObject = (after[0]
      ?.text
      === '{')
      || ((after[0]
        ?.text
        === '(')
        && after.some(function isQuestion(token,): boolean {
          return token.text === '?';
        },));
    return (!inlineObject) || after.some(function namesKey(token,): boolean {
      return token.text === key;
    },);
  }

  /**
   Whether a spread that can replace the value follows the first property under the key.
   */
  const spreadAfter = properties
    .slice(properties.findIndex(isKey,) + 1,)
    .some(mayReplaceValue,)
    && (values.length > 0);
  // A SPREAD ALONE IS NOT NONE (ledger B1150): the role may ride in what it spreads, so a case that expects `none` of such a
  // call would hold vacuously. It reads as `only a spread`, which a case accepts by name.
  if (values.length === 0)
    return properties.some(mayReplaceValue,) ? 'only a spread' : 'none';
  if (values.length > 1)
    return `duplicate ${key}: ${values.join(', ',)}`;
  return spreadAfter ? `${values[0] ?? ''} then a spread` : values[0] ?? '';
}

/**
 Whether the source names a function anywhere outside comments and strings, as a call, a declaration or an import.

 @param source - text of the file

 @param name - function looked for

 @returns True when a word of that name stands in the code

 @example
 ```ts
 const reads = sourceNamesFunction({ source, name: 'gatherStageVoices', },);
 ```
 */
export function sourceNamesFunction(
  {
    source,
    name,
  }: {
    readonly source: string;
    readonly name: string;
  },
): boolean {
  return tokensOf({ source, },)
    .some(function namesIt(token,): boolean {
      return (token.kind === 'word') && (token.text === name);
    },);
}

/**
 Reads the value each real call of a function names under a key.

 @param source - text of the file that calls the function

 @param callee - function called

 @param key - key of the argument object that names the role; `role` unless a stage takes it under another name, and
 dotted (`judging.role`) where it sits inside an object the call passes

 @returns One entry per real call, in source order: the value named, the name of the variable passed, `none` where the
 call names none, `<value> then a spread` where a spread after the key can replace it, `duplicate <key>: a, b` where the
 key is written twice, or `not an object literal` where the argument is not an object literal; empty where the source
 names the function and calls it nowhere

 @throws {@link Error} when the source never names the function, since a test that names the wrong function would
 otherwise read as a file with no call

 @example
 ```ts
 expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual(['wide-judge',],);
 ```
 */
export function rolesNamedAtCalls(
  {
    source,
    callee,
    key = 'role',
  }: {
    readonly source: string;
    readonly callee: string;
    readonly key?: string;
  },
): readonly string[] {
  /**
   Tokens of the file.
   */
  const tokens = tokensOf({ source, },);
  // A CALLEE THE SOURCE NEVER NAMES IS A TEST THAT NAMES THE WRONG FUNCTION, not a file with no call: the list it would
  // return is empty either way, and a test that expects a role then fails with nothing to say why (ledger B1140).
  if (!tokens.some(function namesCallee(token,): boolean {
    return (token.kind === 'word') && (token.text === callee);
  },))
    throw new Error(`the source never names ${callee}, so no call of it can be read`,);

  /**
   What each call names.
   */
  const named: string[] = [];
  for (let at = 0; at < tokens.length; at += 1) {
    if ((tokens[at]
      ?.kind
      !== 'word') || (tokens[at]
        ?.text
        !== callee))
      continue;

    /**
     The token before the name, which tells a call from a declaration or a method of another object.
     */
    const before = tokens[at - 1]
      ?.text;
    if ((before === 'function') || (before === '.'))
      continue;

    /**
     Index of the token after the name and its type arguments.
     */
    const afterName = (tokens[at + 1]
      ?.text
      === '<') ? afterBracket({
        tokens,
        open: at + 1,
        angled: true,
      },) : at + 1;
    if (tokens[afterName]
      ?.text
      !== '(')
      continue;
    if (tokens[afterName + 1]
      ?.text
      !== '{') {
      named.push('not an object literal',);
      continue;
    }

    /**
     Index one past the closing brace of the argument object.
     */
    const afterObject = afterBracket({
      tokens,
      open: afterName + 1,
      angled: false,
    },);
    /**
     Segments of the key, the last of which is the property read.
     */
    const segments = key.split('.',);
    named.push(namedUnderKey({
      properties: propertiesAtPath({
        path: segments.slice(
          0,
          -1,
        ),
        properties: propertiesOf({
          body: tokens.slice(
            afterName + 2,
            afterObject - 1,
          ),
        },),
      },),
      key: segments.at(-1,) ?? key,
    },),);
  }
  return named;
}

/**
 Reads the role an object literal kept in a variable names, for an argument object built once and handed to two calls.

 @param source - text of the file that builds the literal

 @param name - name of the variable that holds it

 @param key - key of the literal that names the role

 @returns Value named under the key as {@link rolesNamedAtCalls} reads it, or `no such literal` where no variable of
 that name is set to an object literal

 @example
 ```ts
 expect(roleNamedInLiteral({ source, name: 'judging', },),).toBe('late-judge',);
 ```
 */
export function roleNamedInLiteral(
  {
    source,
    name,
    key = 'role',
  }: {
    readonly source: string;
    readonly name: string;
    readonly key?: string;
  },
): string {
  /**
   Tokens of the file.
   */
  const tokens = tokensOf({ source, },);

  /**
   Index of the equals sign that sets the variable.
   */
  const equals = tokens.findIndex(function isSetting(
    token,
    at,
  ): boolean {
    return (token.text === name)
      && DECLARING_WORDS.has(tokens[at - 1]
        ?.text
        ?? '',)
      && (tokens.slice(
        at,
        at + SETTING_REACH,
      )
        .some(function isEquals(later,): boolean {
          return later.text === '=';
        },));
  },);
  if (tokens[equals]
    ?.text
    !== name)
    return 'no such literal';

  /**
   Index of the opening brace after the equals sign.
   */
  const open = tokens.findIndex(function isOpening(
    token,
    at,
  ): boolean {
    return (at > equals) && (token.text === '{');
  },);
  /**
   Index one past the closing brace of the literal.
   */
  const after = afterBracket({
    tokens,
    open,
    angled: false,
  },);
  return namedUnderKey({
    properties: propertiesOf({
      body: tokens.slice(
        open + 1,
        after - 1,
      ),
    },),
    key,
  },);
}

//endregion Re-ask role at a call

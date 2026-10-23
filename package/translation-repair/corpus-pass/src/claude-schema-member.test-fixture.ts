import {
  isIdentifierCharacter,
  leadingNameOf,
  positionsWhere,
  startsAsFunctionValue,
} from './claude-schema-text.test-fixture.ts';

//region Functions that are members of an object literal
// What `claude-schema-code.test-fixture.ts` reads when a top-level constant holds an object literal: the members one
// level in that are functions, a method (`name(...) {`, `async name(...) {`) or a property whose value starts as a
// function does (`name: (...) => ...`, `name: function ...`). A member deeper in the object, an accessor, a computed
// name and a member on the same line as the opening brace are not read.

/**
 A function that is a member of an object literal.
 */
type MemberFunction = {
  /**
   The member's name behind the object's: `object.member`.
   */
  readonly name: string;

  /**
   The member's lines, joined.
   */
  readonly text: string;
};

/**
 What marks an asynchronous method before its name.
 */
const ASYNC_PREFIX = 'async ';

/**
 The name of the function a member's text opens, empty when the member is no function.

 @param text - member's text, from the line that opens it

 @returns Member's name

 @example
 ```ts
 const name = memberNameOf({ text: '  async build(a: string,) {\n    return a;\n  },', },);
 // => 'build'
 ```
 */
function memberNameOf({ text, }: { readonly text: string; },): string {
  /**
   The text without its indentation.
   */
  const trimmed = text.trimStart();
  /**
   Whether the text starts with the mark of an asynchronous method and a name after it, which `async(...) {` is not.
   */
  const marksAsync = trimmed.startsWith(ASYNC_PREFIX,)
    && isIdentifierCharacter(trimmed.at(ASYNC_PREFIX.length,) ?? '',);
  /**
   The text without the mark of an asynchronous method.
   */
  const withoutAsync = marksAsync ? trimmed.slice(ASYNC_PREFIX.length,) : trimmed;
  /**
   The member's name.
   */
  const name = leadingNameOf({ text: withoutAsync, },);
  /**
   What follows the name.
   */
  const afterName = withoutAsync.slice(name.length,)
    .trimStart();
  if (afterName.startsWith('(',) || afterName.startsWith('<',))
    return name;
  /**
   What follows the colon of a property.
   */
  const afterColon = afterName.slice(1,)
    .trimStart();
  return (afterName.startsWith(':',) && startsAsFunctionValue({ value: afterColon, },))
    ? name
    : '';
}

/**
 How many characters of indentation a line has.

 @param line - one line of code

 @returns Characters before the first one that is no blank

 @example
 ```ts
 const indentation = indentationOf({ line: '  build() {', },);
 // => 2
 ```
 */
function indentationOf({ line, }: { readonly line: string; },): number {
  /**
   The line without its indentation.
   */
  const trimmed = line.trimStart();
  return line.length - trimmed.length;
}

/**
 Whether a line starts with a name, which a member of an object does.

 @param line - one line of code

 @returns Whether the first character after the indentation is one a name holds

 @example
 ```ts
 const starts = startsWithName({ line: '  build() {', },);
 // => true
 ```
 */
function startsWithName({ line, }: { readonly line: string; },): boolean {
  /**
   The line without its indentation.
   */
  const trimmed = line.trimStart();
  return isIdentifierCharacter(trimmed.at(0,) ?? '',);
}

/**
 The functions among the members of an object literal, one level in.

 @param owner - name of the constant that holds the object

 @param lines - declaration's lines, the first of which opens the object

 @returns One entry per member that is a function, in order

 @example
 ```ts
 const lines = ['export const kit = {', '  build() {', '    return 1;', '  },', '};',];
 const members = memberFunctionsOf({ owner: 'kit', lines, },);
 // => [{ name: 'kit.build', text: '  build() {\n    return 1;\n  },' }]
 ```
 */
export function memberFunctionsOf(
  {
    owner,
    lines,
  }: {
    readonly owner: string;
    readonly lines: readonly string[];
  },
): readonly MemberFunction[] {
  /**
   The lines inside the object, after the line that opens it.
   */
  const body = lines.slice(1,);
  /**
   The first line that holds anything, whose indentation is the members'.
   */
  const first = body.find(function holdsText(line,): boolean {
    return line.trim() !== '';
  },) ?? '';
  /**
   How many characters of indentation a member's first line has.
   */
  const indentation = indentationOf({ line: first, },);
  /**
   Positions of the lines that start a member: at the members' indentation, beginning with a name.
   */
  const starts = positionsWhere({
    lines: body,
    test: function startsMember(line,): boolean {
      return (indentationOf({ line, },) === indentation) && startsWithName({ line, },);
    },
  },);
  return starts.flatMap(function memberFrom(
    start,
    index,
  ): readonly MemberFunction[] {
    /**
     The member's text: from its first line up to the next member or the end of the object.
     */
    const text = body.slice(
      start,
      starts[index + 1] ?? body.length,
    )
      .join('\n',);
    /**
     The member's name, empty when the member is no function.
     */
    const name = memberNameOf({ text, },);
    return (name === '')
      ? []
      : [
        {
          name: `${owner}.${name}`,
          text,
        },
      ];
  },);
}

//endregion Functions that are members of an object literal

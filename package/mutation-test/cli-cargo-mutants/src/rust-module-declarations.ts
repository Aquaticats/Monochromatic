/**
 The `mod name;` declarations of one Rust source file, read from its tokens.
 */
import {
  isPunct,
  type RustToken,
} from './rust-tokens.ts';

/**
 Distance from the `path` word of an attribute to the string that follows its equals sign.
 */
const pathValueOffset = 3;

/**
 A `mod name;` declaration that names a file.
 */
export type ModuleDeclaration = {
  readonly name: string;
  /**
   The `#[path = "..."]` attribute in front of the declaration, absent when the file follows from the module name.
   */
  readonly path?: string;
  /**
   The paths of `#[cfg_attr(condition, path = "...")]` attributes in front of the declaration, absent when there are
   none. Which one applies depends on a condition the scan does not evaluate, so every one that exists is followed,
   beside the file the module name gives.
   */
  readonly conditionalPaths?: readonly string[];
  /**
   The inline modules (`mod outer { ... }`) that enclose the declaration, outermost first, absent at the file's top level.
   */
  readonly inline?: readonly string[];
};

/**
 Find the index of the bracket that closes the one at a position.

 @param tokens - source tokens

 @param open - index of an opening `[`

 @returns the index of the matching `]`, or the token count when it is unterminated

 @example
 ```ts
 closingBracket({ tokens, open: 1 });
 ```
 */
function closingBracket({
  tokens,
  open,
}: {
  readonly tokens: readonly RustToken[];
  readonly open: number;
}): number {
  for (let depth = 0, index = open; index < tokens.length; index += 1) {
    if (isPunct({
      tokens,
      index,
      text: '[',
    }))
      depth += 1;
    else if (isPunct({
      tokens,
      index,
      text: ']',
    })) {
      depth -= 1;
      if (depth === 0)
        return index;
    }
  }
  return tokens.length;
}

/**
 Find the paths a `cfg_attr` attribute gives to the module it precedes.

 @param tokens - source tokens

 @param open - index of the attribute's opening bracket

 @param close - index of its closing bracket

 @returns the strings that follow `path =` inside it, in order

 @example
 ```ts
 conditionalPathsIn({ tokens, open: 1, close: 12 });
 ```
 */
function conditionalPathsIn({
  tokens,
  open,
  close,
}: {
  readonly tokens: readonly RustToken[];
  readonly open: number;
  readonly close: number;
}): readonly string[] {
  if (tokens[open + 1]
    ?.text
    !== 'cfg_attr')
    return [];
  /**
   The attribute's tokens after its name.
   */
  const inside = tokens.slice(
    open + 1,
    close,
  );
  return inside.flatMap(function pathValue(
    token,
    index
  ) {
    /**
     The string after `path =`, when this token is the word `path`.
     */
    const value = inside[(index + pathValueOffset) - 1];
    return (token.kind === 'ident') && (token.text === 'path')
      && isPunct({
        tokens: inside,
        index: index + 1,
        text: '=',
      })
      && (value?.kind === 'string') ? [value.text] : [];
  });
}

/**
 List the `mod name;` declarations of a file, each with its `#[path]` attributes and its enclosing inline modules.

 Inline modules (`mod name { ... }`) are not declarations of a file and are not listed; the declarations inside them are,
 with the names of the inline modules around them, because their files sit below those modules' directories.

 @param tokens - source tokens

 @returns the declarations in source order

 @example
 ```ts
 moduleDeclarations(tokenizeRust('#[path = "x_tests.rs"] mod tests;'));
 ```
 */
export function moduleDeclarations(tokens: readonly RustToken[]): readonly ModuleDeclaration[] {
  /**
   Declarations found so far.
   */
  const declarations: ModuleDeclaration[] = [];
  /**
   Attributes seen since the last item boundary: the direct path (empty, or one text) and the conditional ones.
   */
  const pending: {
    readonly path: string[];
    readonly conditional: string[];
  } = {
    path: [],
    conditional: [],
  };
  /**
   One entry per open brace: the name of the inline module it opens, or the empty text for any other block.
   */
  const blocks: string[] = [];
  /**
   Name of the inline module whose opening brace comes next, empty or one name.
   */
  const opening: string[] = [];
  /**
   Forget the attributes seen so far.
   */
  function clearPending(): void {
    pending.path
      .length = 0;
    pending.conditional
      .length = 0;
  }
  for (let index = 0; index < tokens.length;) {
    /**
     The token at the index.
     */
    const token = tokens[index];
    /**
     Index of the opening bracket of an attribute that starts here, or of nothing.
     */
    const open = index + (isPunct({
      tokens,
      index: index + 1,
      text: '!',
    }) ? 2 : 1);
    if (isPunct({
      tokens,
      index,
      text: '#',
    }) && isPunct({
      tokens,
      index: open,
      text: '[',
    })) {
      /**
       Index of the attribute's closing bracket.
       */
      const close = closingBracket({
        tokens,
        open,
      });
      if ((tokens[open + 1]
        ?.text
        === 'path') && isPunct({
          tokens,
          index: open + 2,
          text: '=',
        })
        && (tokens[open + pathValueOffset]
          ?.kind
          === 'string'))
        pending.path
          .splice(
          0,
          pending.path
            .length,
          tokens[open + pathValueOffset]
            ?.text
            ?? '',
          );
      pending.conditional
        .push(...conditionalPathsIn({
          tokens,
          open,
          close,
        }));
      index = close + 1;
    }
    else if ((token?.kind === 'ident') && (token.text === 'mod')
      && (tokens[index + 1]
        ?.kind
        === 'ident')) {
      /**
       The module's name.
       */
      const name = tokens[index + 1]
        ?.text
        ?? '';
      if (isPunct({
        tokens,
        index: index + 2,
        text: ';',
      })) {
        declarations.push({
          name,
          ...(pending.path
            .length
            === 0 ? {} : { path: pending.path[0] ?? '' }),
          ...(pending.conditional
            .length
            === 0 ? {} : { conditionalPaths: [...pending.conditional] }),
          ...(blocks.some(Boolean) ? { inline: blocks.filter(Boolean) } : {}),
        });
      }
      else if (isPunct({
        tokens,
        index: index + 2,
        text: '{',
      }))
        opening.splice(
          0,
          opening.length,
          name,
        );
      clearPending();
      index += 2;
    }
    else {
      if (isPunct({
        tokens,
        index,
        text: '{',
      }))
        blocks.push(opening.shift() ?? '');
      else if (isPunct({
        tokens,
        index,
        text: '}',
      }))
        blocks.pop();
      if ([
        ';',
        '{',
        '}'
      ].some(function isBoundary(text) {
        return isPunct({
          tokens,
          index,
          text,
        });
      }))
        clearPending();
      index += 1;
    }
  }
  return declarations;
}

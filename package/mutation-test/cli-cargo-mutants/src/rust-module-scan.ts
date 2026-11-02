/**
 What the module and import scanners read out of the tokens of one Rust source file.
 */
import { type RustToken } from './rust-tokens.ts';

/**
 A `mod name;` declaration that names a file.
 */
export type ModuleDeclaration = {
  readonly name: string;
  /**
   The `#[path = "..."]` attribute in front of the declaration, absent when the file follows from the module name.
   */
  readonly path?: string;
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
  /**
   Open brackets, counting the one at the start.
   */
  let depth = 0;
  for (let index = open; index < tokens.length; index += 1) {
    if (tokens[index]?.text === '[')
      depth += 1;
    else if (tokens[index]?.text === ']') {
      depth -= 1;
      if (depth === 0)
        return index;
    }
  }
  return tokens.length;
}

/**
 List the `mod name;` declarations of a file, each with its `#[path]` attribute.

 Inline modules (`mod name { ... }`) are not declarations of a file and are not listed; the declarations inside them are
 listed as if they stood in the file itself.

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
   The `path` attribute seen since the last item boundary.
   */
  let pendingPath: string | undefined;
  /**
   Index being read.
   */
  let index = 0;
  while (index < tokens.length) {
    /**
     The token at the index.
     */
    const token = tokens[index];
    if ((token?.text === '#') && (tokens[index + (tokens[index + 1]?.text === '!' ? 2 : 1)]?.text === '[')) {
      /**
       Index of the opening bracket of the attribute.
       */
      const open = index + (tokens[index + 1]?.text === '!' ? 2 : 1);
      /**
       Index of its closing bracket.
       */
      const close = closingBracket({
        tokens,
        open,
      });
      if ((tokens[open + 1]?.text === 'path') && (tokens[open + 2]?.text === '=') && (tokens[open + 3]?.kind === 'string'))
        pendingPath = tokens[open + 3]?.text;
      index = close + 1;
    }
    else if ((token?.kind === 'ident') && (token.text === 'mod') && (tokens[index + 1]?.kind === 'ident')) {
      if (tokens[index + 2]?.text === ';') {
        declarations.push({
          name: tokens[index + 1]?.text ?? '',
          ...(pendingPath === undefined ? {} : { path: pendingPath }),
        });
      }
      pendingPath = undefined;
      index += 2;
    }
    else {
      if ((token?.text === ';') || (token?.text === '{') || (token?.text === '}'))
        pendingPath = undefined;
      index += 1;
    }
  }
  return declarations;
}

/**
 Add the names a `use` group opens, at its first nesting level.

 @param tokens - source tokens

 @param open - index of the opening brace

 @param names - set that receives each first path segment

 @example
 ```ts
 addGroupNames({ tokens, open: 3, names });
 ```
 */
function addGroupNames({
  tokens,
  open,
  names,
}: {
  readonly tokens: readonly RustToken[];
  readonly open: number;
  readonly names: Set<string>;
}): void {
  /**
   Open braces, counting the one at the start.
   */
  let depth = 0;
  /**
   Whether the next identifier at the first level starts a group item.
   */
  let itemStart = false;
  for (let index = open; (index < tokens.length) && (depth >= 0); index += 1) {
    /**
     The token at the index.
     */
    const token = tokens[index];
    if (token?.text === '{') {
      depth += 1;
      itemStart = depth === 1;
    }
    else if (token?.text === '}') {
      depth -= 1;
      if (depth === 0)
        return;
    }
    else if ((token?.text === ',') && (depth === 1))
      itemStart = true;
    else if (itemStart && (token?.kind === 'ident') && (depth === 1)) {
      names.add(token.text);
      itemStart = false;
    }
    else
      itemStart = false;
  }
}

/**
 List the names a file reaches through a path that starts at one of the given roots.

 A path such as `crate::repo_read_query::answer_with`, `super::repo_read_query::{Answer, Query}` or
 `super::super::repo_read_query` names `repo_read_query`; a group such as `use crate::{a::f, b}` names `a` and `b`.
 Every identifier that follows a root is listed, whether it is a module or an item, so the caller keeps the ones that
 are modules. The scan over-approximates: a name that merely appears after a root is listed.

 @param tokens - source tokens

 @param roots - identifiers that start a path into the crate, such as `crate`, `super` and the library's own name

 @returns the distinct names

 @example
 ```ts
 referencedNames({ tokens: tokenizeRust('use super::a::b;'), roots: new Set(['super']) });
 ```
 */
export function referencedNames({
  tokens,
  roots,
}: {
  readonly tokens: readonly RustToken[];
  readonly roots: ReadonlySet<string>;
}): ReadonlySet<string> {
  /**
   Names found so far.
   */
  const names = new Set<string>();
  for (let index = 0; index < tokens.length; index += 1) {
    /**
     The token at the index.
     */
    const token = tokens[index];
    if ((token?.kind !== 'ident') || (!roots.has(token.text)) || (tokens[index + 1]?.text !== '::'))
      continue;
    /**
     Index of the token after the root and its separator.
     */
    let next = index + 2;
    while ((tokens[next]?.text === 'super') && (tokens[next + 1]?.text === '::'))
      next += 2;
    /**
     The token that names the first segment after the root.
     */
    const first = tokens[next];
    if (first?.text === '{') {
      addGroupNames({
        tokens,
        open: next,
        names,
      });
    }
    else if (first?.kind === 'ident')
      names.add(first.text);
  }
  names.delete('self');
  names.delete('super');
  names.delete('crate');
  return names;
}

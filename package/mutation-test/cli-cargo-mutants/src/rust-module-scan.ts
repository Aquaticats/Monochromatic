/**
 What the import scanners read out of the tokens of one Rust source file: the modules it refers to, and what a root module
 re-exports.
 */
import {
  isPunct,
  type RustToken,
} from './rust-tokens.ts';

/**
 Name that stands for every module of the crate in the names a file refers to: the file includes source the scan does not
 read, so any module may be reached through it.
 */
export const everyModule = '*';

/**
 Add the names a `use` group opens, at its first nesting level.

 @param tokens - source tokens

 @param open - index of the opening brace

 @param names - set that receives each first path segment

 @returns true when the group lists a glob (`*`) at its first level

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
}): boolean {
  /**
   Open braces, counting the one at the start, whether the next identifier at the first level starts a group item, and
   whether a glob was listed.
   */
  const state = {
    depth: 0,
    itemStart: false,
    glob: false,
  };
  for (let index = open; index < tokens.length; index += 1) {
    /**
     The token at the index.
     */
    const token = tokens[index];
    if (isPunct({
      tokens,
      index,
      text: '{',
    })) {
      state.depth += 1;
      state.itemStart = state.depth === 1;
    }
    else if (isPunct({
      tokens,
      index,
      text: '}',
    })) {
      state.depth -= 1;
      if (state.depth === 0)
        return state.glob;
    }
    else if (isPunct({
      tokens,
      index,
      text: ',',
    }) && (state.depth === 1))
      state.itemStart = true;
    else if (state.itemStart && (token?.kind === 'ident')
      && (state.depth === 1)) {
      names.add(token.text);
      state.itemStart = false;
    }
    else {
      state.glob ||= isPunct({
        tokens,
        index,
        text: '*',
      }) && (state.depth === 1);
      state.itemStart = false;
    }
  }
  return state.glob;
}

/**
 List the names a file reaches through a path that starts at one of the given roots.

 A path such as `crate::repo_read_query::answer_with`, `super::repo_read_query::{Answer, Query}` or
 `super::super::repo_read_query` names `repo_read_query`; a group such as `use crate::{a::f, b}` names `a` and `b`.
 Every identifier that follows a root is listed, whether it is a module or an item, so the caller keeps the ones that
 are modules and the items the crate root re-exports. The scan over-approximates: a name that merely appears after a
 root is listed.

 Two things reach modules without naming them. A glob of the crate root (`use crate::*`, or `use super::*` in a module
 directly below the root) brings every module into scope, so every identifier of the file is listed. `include!` pulls in
 source the scan does not read, so {@link everyModule} is listed.

 @param tokens - source tokens

 @param roots - identifiers that start a path into the crate, such as `crate`, `super` and the library's own name

 @param depth - how many modules the file sits below the crate root, which decides which `super` paths end at the root

 @returns the distinct names

 @example
 ```ts
 referencedNames({ tokens: tokenizeRust('use super::a::b;'), roots: new Set(['super']), depth: 1 });
 ```
 */
export function referencedNames({
  tokens,
  roots,
  depth,
}: {
  readonly tokens: readonly RustToken[];
  readonly roots: ReadonlySet<string>;
  readonly depth: number;
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
    if ((token?.kind === 'ident') && (token.text === 'include')
      && isPunct({
        tokens,
        index: index + 1,
        text: '!',
      }))
      names.add(everyModule);
    if ((token?.kind !== 'ident') || (!roots.has(token.text))
      || (tokens[index + 1]
        ?.text
        !== '::'))
      continue;
    /**
     Index of the token after the root and its separator.
     */
    let next = index + 2;
    /**
     How many `super` steps the path takes.
     */
    let steps = token.text === 'super' ? 1 : 0;
    while ((tokens[next]
      ?.text
      === 'super') && (tokens[next + 1]
        ?.text
        === '::')) {
      next += 2;
      steps += 1;
    }
    /**
     The token that names the first segment after the root.
     */
    const first = tokens[next];
    /**
     Whether the path ends at the crate root: `crate`, the library's name, or as many `super` steps as the file is deep.
     */
    const atRoot = token.text === 'super' ? steps === depth : token.text !== 'self';
    /**
     Whether the path imports everything the root holds.
     */
    const glob = isPunct({
      tokens,
      index: next,
      text: '*',
    }) || (isPunct({
      tokens,
      index: next,
      text: '{',
    }) && addGroupNames({
      tokens,
      open: next,
      names,
    }));
    if (atRoot && glob) {
      for (const each of tokens) {
        if (each.kind === 'ident')
          names.add(each.text);
      }
    }
    else if (first?.kind === 'ident')
      names.add(first.text);
  }
  names.delete('self');
  names.delete('super');
  names.delete('crate');
  return names;
}

/**
 List what a root module re-exports with `use`, by the module each name may come from.

 Every `use` statement is read as `use [crate::|self::]module::rest`: each identifier in `rest` is an item that may come
 from `module`, and a glob in `rest` makes `module` a source of any item, listed under `*`. It over-approximates.

 @param tokens - source tokens of the root module

 @returns for each item name (and `*`), the modules it may come from

 @example
 ```ts
 rootReexports(tokenizeRust('pub use a::value;'));
 ```
 */
export function rootReexports(tokens: readonly RustToken[]): ReadonlyMap<string, readonly string[]> {
  /**
   Item names and the modules they may come from.
   */
  const sources = new Map<string, readonly string[]>();
  for (let index = 0; index < tokens.length; index += 1) {
    if ((tokens[index]
      ?.kind
      !== 'ident') || (tokens[index]
        ?.text
        !== 'use'))
      continue;
    /**
     Index of the statement's `;`, or minus one when it has none.
     */
    const end = tokens.findIndex(function endsStatement(
      _token,
      at
    ) {
      return (at > index) && isPunct({
        tokens,
        index: at,
        text: ';',
      });
    });
    /**
     The statement's tokens after `use`, without the path separators.
     */
    const statement = tokens.slice(
      index + 1,
      end === (-1) ? tokens.length : end,
    )
      .filter(function isPathPart(token) {
        return !((token.kind === 'punct') && (token.text === '::'));
      });
    /**
     The statement without a leading `crate` or `self`.
     */
    const rest = [
      'crate',
      'self'
    ].includes(statement[0]
      ?.text
      ?? '') ? statement.slice(1) : statement;
    /**
     The module the statement takes names from, and the tokens after it.
     */
    const [module, ...items] = rest;
    if (module?.kind !== 'ident')
      continue;
    for (const item of items) {
      if (item.kind === 'ident') {
        if (![
          'self',
          'as'
        ].includes(item.text)) {
          sources.set(
            item.text,
            [
              ...sources.get(item.text) ?? [],
              module.text
            ],
          );
        }
      }
      else if ((item.kind === 'punct') && (item.text === '*')) {
        sources.set(
          everyModule,
          [
            ...sources.get(everyModule) ?? [],
            module.text
          ],
        );
      }
    }
  }
  return sources;
}

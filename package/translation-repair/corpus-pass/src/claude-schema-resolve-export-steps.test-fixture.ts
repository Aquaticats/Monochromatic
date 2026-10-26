import { EXPORT_ASSIGNMENT, } from './claude-schema-equals.test-fixture.ts';
import type {
  Family,
  Resolved,
} from './claude-schema-resolve.test-fixture.ts';
import {
  NONE,
  OPAQUE,
  type Resolve,
} from './claude-schema-resolve-steps.test-fixture.ts';
import { targetOf, } from './claude-schema-target.test-fixture.ts';

//region The steps that read an export of a module
// What `claude-schema-resolve.test-fixture.ts` dispatches the questions about an export to: what a module exports
// under a name, followed through the modules it exports from, and what it exports through the modules it exports
// everything of. They ask the questions they lead to through `resolve`, as the steps in
// `claude-schema-resolve-steps.test-fixture.ts` do.

/**
 What a module exports under a name, followed through the modules it exports from.

 @param family - modules of the sources

 @param file - path of the module

 @param name - name it exports, or `default`

 @param seen - exports already followed, so a module that exports itself ends

 @param resolve - answers the questions the export leads to

 @returns What the export stands for, `none` when the module exports no such name

 @example
 ```ts
 const resolved = exportStep({ family, file: 'a/src/index.ts', name: 'build', seen: new Set<string>(), resolve, },);
 ```
 */
export function exportStep(
  {
    family,
    file,
    name,
    seen,
    resolve,
  }: {
    readonly family: Family;
    readonly file: string;
    readonly name: string;
    readonly seen: ReadonlySet<string>;
    readonly resolve: Resolve;
  },
): Resolved {
  /**
   The export as a place to stop when the walk comes back to it.
   */
  const place = `${file}#export:${name}`;
  /**
   What the module exports, absent when the file is none of the sources.
   */
  const exports = family.get(file,)
    ?.exports;
  /**
   What the module exports under the name, absent when it exports none; the `export =` of a module gives its default.
   */
  const entry = exports?.get(name,) ?? ((name === 'default') ? exports?.get(EXPORT_ASSIGNMENT,) : undefined);
  if (seen.has(place,))
    return NONE;
  /**
   The exports followed, this one included.
   */
  const followed = new Set([
    ...seen,
    place,
  ],);
  if (entry === undefined) {
    return resolve({
      family,
      seen: followed,
      query: {
        kind: 'star',
        file,
        name,
      },
    },);
  }
  if (entry.kind === 'local') {
    /**
     What the local name stands for.
     */
    const local = resolve({
      family,
      seen: followed,
      query: {
        kind: 'name',
        file,
        name: entry.name,
      },
    },);
    return (local.kind === 'free') ? OPAQUE : local;
  }
  /**
   The module the entry exports from, empty when it is outside the sources.
   */
  const target = targetOf({
    family,
    from: file,
    specifier: entry.specifier,
  },);
  if (entry.kind === 'namespace') {
    return (target === '')
      ? { kind: 'outside', }
      : {
        kind: 'namespace',
        file: target,
      };
  }
  return (target === '')
    ? {
      kind: 'free',
      name: entry.imported,
    }
    : resolve({
      family,
      seen: followed,
      query: {
        kind: 'export',
        file: target,
        name: entry.imported,
      },
    },);
}

/**
 What a module exports under a name that is none of its own entries: what the modules it exports everything of export,
 the first that exports the name winning, and never the default.

 @param family - modules of the sources

 @param file - path of the module

 @param name - name it exports

 @param seen - exports already followed

 @param resolve - answers the questions the export leads to

 @returns What the export stands for, `none` when no module it exports everything of exports the name

 @example
 ```ts
 const resolved = starStep({ family, file, name: 'build', seen: new Set<string>(), resolve, },);
 ```
 */
export function starStep(
  {
    family,
    file,
    name,
    seen,
    resolve,
  }: {
    readonly family: Family;
    readonly file: string;
    readonly name: string;
    readonly seen: ReadonlySet<string>;
    readonly resolve: Resolve;
  },
): Resolved {
  if (name === 'default')
    return NONE;
  for (const specifier of family.get(file,)
    ?.stars
    ?? []) {
    /**
     The module the star exports, empty when it is outside the sources.
     */
    const target = targetOf({
      family,
      from: file,
      specifier,
    },);
    /**
     What that module exports under the name.
     */
    const resolved = (target === '')
      ? NONE
      : resolve({
        family,
        seen,
        query: {
          kind: 'export',
          file: target,
          name,
        },
      },);
    if (resolved.kind !== 'none')
      return resolved;
  }
  return NONE;
}

//endregion The steps that read an export of a module

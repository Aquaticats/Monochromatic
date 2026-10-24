import {
  type Family,
  resolveExport,
} from './claude-schema-resolve.test-fixture.ts';
import type { SchemaBuilder, } from './claude-schema-sources.test-fixture.ts';
import { targetOf, } from './claude-schema-target.test-fixture.ts';
import {
  keyOf,
  type KnownFunctions,
} from './claude-schema-usage.test-fixture.ts';

//region Which modules carry a builder
// What the check behind the schema search (`claude-schema-blind-spots.test-fixture.ts`) reads a path against: the
// modules a path can lead to a builder through. A module carries a builder when it declares one, or when something it
// exports is one: an export of its own, one it re-exports from another module, a namespace of a module that carries
// one, or an object that holds one as a member.

/**
 The nothing followed when an export is resolved from the start.
 */
const NOTHING_FOLLOWED: ReadonlySet<string> = new Set<string>();

/**
 The objects that hold a builder as a member, by the module that declares them and their name.

 @param builders - builders and wrappers of the sources

 @returns Keys of the objects, `file#object`

 @example
 ```ts
 const objects = carryingObjectsOf({ builders, },);
 ```
 */
export function carryingObjectsOf(
  {
    builders,
  }: {
    readonly builders: readonly SchemaBuilder[];
  },
): ReadonlySet<string> {
  return new Set(builders.flatMap(function ownerOf({
    name,
    file,
  },): readonly string[] {
    /**
     Where the owner's name ends in `owner.member`.
     */
    const dot = name.indexOf('.',);
    return (dot === (-1))
      ? []
      : [
        keyOf({
          file,
          name: name.slice(
            0,
            dot,
          ),
        },),
      ];
  },),);
}

/**
 Every name a module exports, its own and those of the modules it exports everything of, never the default through them.

 @param family - modules of the sources

 @param file - path of the module

 @param seen - modules already read, so two modules that export everything of each other end

 @returns Names, each once

 @example
 ```ts
 const names = exportNamesOf({ family, file: 'a/src/index.ts', seen: new Set<string>(), },);
 ```
 */
function exportNamesOf(
  {
    family,
    file,
    seen,
  }: {
    readonly family: Family;
    readonly file: string;
    readonly seen: ReadonlySet<string>;
  },
): readonly string[] {
  /**
   The module read, absent when the sources hold none of that path.
   */
  const facts = family.get(file,);
  if ((facts === undefined) || seen.has(file,))
    return [];
  /**
   The modules read, this one included.
   */
  const followed = new Set([
    ...seen,
    file,
  ],);
  return [
    ...new Set([
      ...facts.exports
        .keys(),
      ...facts.stars
        .flatMap(function starredNames(specifier,): readonly string[] {
        /**
         The module the star exports, empty when it is outside the sources.
         */
          const target = targetOf({
            family,
            from: file,
            specifier,
          },);
          return (target === '')
            ? []
            : exportNamesOf({
              family,
              file: target,
              seen: followed,
            },)
              .filter(function isNotDefault(name,): boolean {
                return name !== 'default';
              },);
        },),
    ],),
  ];
}

/**
 Whether something a module exports is a builder, a namespace of a module already known to carry one, or an object that
 holds one.

 @param family - modules of the sources

 @param file - path of the module

 @param known - builders and wrappers of the sources

 @param bearing - modules already known to carry a builder

 @param carriers - objects that hold a builder

 @returns Whether some export of the module leads to a builder

 @example
 ```ts
 const carries = exportsBuilder({ family, file, known, bearing, carriers, },);
 ```
 */
function exportsBuilder(
  {
    family,
    file,
    known,
    bearing,
    carriers,
  }: {
    readonly family: Family;
    readonly file: string;
    readonly known: KnownFunctions;
    readonly bearing: ReadonlySet<string>;
    readonly carriers: ReadonlySet<string>;
  },
): boolean {
  return exportNamesOf({
    family,
    file,
    seen: NOTHING_FOLLOWED,
  },)
    .some(function leadsToBuilder(name,): boolean {
      /**
       What the module exports under the name.
       */
      const resolved = resolveExport({
        family,
        file,
        name,
        seen: NOTHING_FOLLOWED,
      },);
      if (resolved.kind === 'function')
        return known.keys
          .has(keyOf(resolved,),);
      if (resolved.kind === 'namespace')
        return bearing.has(resolved.file,);
      return (resolved.kind === 'object') && carriers.has(keyOf(resolved,),);
    },);
}

/**
 The modules that carry a builder, until no module is added.

 @param family - modules of the sources

 @param builders - builders and wrappers of the sources

 @returns Paths of the modules

 @example
 ```ts
 const bearing = bearingFilesOf({ family, builders, },);
 ```
 */
export function bearingFilesOf(
  {
    family,
    builders,
  }: {
    readonly family: Family;
    readonly builders: readonly SchemaBuilder[];
  },
): ReadonlySet<string> {
  /**
   The builders by where each sits and by name.
   */
  const known: KnownFunctions = {
    keys: new Set(builders.map(function keyOfBuilder(builder,): string {
      return keyOf(builder,);
    },),),
    names: new Set(builders.map(function nameOfBuilder({ name, },): string {
      return name;
    },),),
  };
  /**
   The objects that hold a builder.
   */
  const carriers = carryingObjectsOf({ builders, },);

  /**
   The modules that carry a builder, grown one pass at a time.

   @param bearing - modules known so far

   @returns Modules known after the pass that adds none
   */
  function grown(bearing: ReadonlySet<string>,): ReadonlySet<string> {
    /**
     The modules that carry a builder through something they export and are not known yet.
     */
    const added = [...family.keys(),].filter(function isAdded(file,): boolean {
      return (!bearing.has(file,)) && exportsBuilder({
        family,
        file,
        known,
        bearing,
        carriers,
      },);
    },);
    return (added.length === 0)
      ? bearing
      : grown(new Set([
        ...bearing,
        ...added,
      ],),);
  }

  return grown(
    new Set(builders.map(function fileOfBuilder({ file, },): string {
      return file;
    },),),
  );
}

//endregion Which modules carry a builder

import type { ModuleFacts, } from './claude-schema-facts.test-fixture.ts';
import { targetOf, } from './claude-schema-target.test-fixture.ts';

//region Which function a name stands for
// What `claude-schema-sources.test-fixture.ts` asks to tell whether a file uses a builder: for a name or a chain of
// members written in one file, the function it stands for, found by following the file's imports, aliases, destructured
// members and dynamic imports to the module that declares it, and through the modules that export it again. Modules are
// the files handed in; a specifier that names none of them (a package outside the family, a path nobody supplied)
// leaves the name to be read as the function of that name.

/**
 What a name or a chain of members stands for.
 */
export type Resolved =
  | {
    /**
     A function a module declares, or a member of an object it declares as `object.member`.
     */
    readonly kind: 'function';

    /**
     Path of the module.
     */
    readonly file: string;

    /**
     The function's name.
     */
    readonly name: string;
  }
    | {
    /**
     An object a module declares, whose members are functions.
     */
      readonly kind: 'object';

    /**
     Path of the module.
     */
      readonly file: string;

    /**
     The object's name.
     */
      readonly name: string;
    }
    | {
    /**
     A module of the sources as a whole.
     */
      readonly kind: 'namespace';

    /**
     Path of the module.
     */
      readonly file: string;
    }
    | {
    /**
     A name the file neither declares nor imports from a module of the sources, read as the function of that name.
     */
      readonly kind: 'free';

    /**
     The name, a chain of members joined by dots.
     */
      readonly name: string;
    }
    | {
    /**
     A module outside the sources as a whole, whose members are read as free names.
     */
      readonly kind: 'outside';
    }
    | {
    /**
     Something no function is read through: a value that is none, a member of a function.
     */
      readonly kind: 'opaque';
    }
    | {
    /**
     What a module exports under a name it does not export.
     */
      readonly kind: 'none';
    };

/**
 The modules of the sources, by path.
 */
export type Family = ReadonlyMap<string, ModuleFacts>;

/**
 The thing no function is read through.
 */
const OPAQUE: Resolved = { kind: 'opaque', };

/**
 What a module exports under a name it does not export.
 */
const NONE: Resolved = { kind: 'none', };

/**
 What a name that leads to an export reads as, once an export that is not there is told from a value no function is read
 through.

 @param resolved - what the export resolved to

 @returns The thing no function is read through for an export that is not there, the resolved thing otherwise

 @example
 ```ts
 const read = valueOf({ resolved: NONE, },);
 // => { kind: 'opaque' }
 ```
 */
function valueOf({ resolved, }: { readonly resolved: Resolved; },): Resolved {
  return (resolved.kind === 'none') ? OPAQUE : resolved;
}

/**
 What a member read off a thing stands for.

 @param family - the modules of the sources

 @param of - the thing

 @param member - the member's name

 @param seen - the bindings and exports already followed

 @returns The member: a member of an object is the function `object.member` when the module declares one, an export of a
 module is what the module exports, any member of an outside module is a free name, any member of a free name extends it

 @example
 ```ts
 const next = memberOf({ family, of: { kind: 'free', name: 'kit', }, member: 'build', seen: new Set<string>(), },);
 // => { kind: 'free', name: 'kit.build' }
 ```
 */
function memberOf(
  {
    family,
    of,
    member,
    seen,
  }: {
    readonly family: Family;
    readonly of: Resolved;
    readonly member: string;
    readonly seen: ReadonlySet<string>;
  },
): Resolved {
  if (of.kind === 'free') {
    return {
      kind: 'free',
      name: `${of.name}.${member}`,
    };
  }
  if (of.kind === 'outside') {
    return {
      kind: 'free',
      name: member,
    };
  }
  if (of.kind === 'namespace') {
    return valueOf({
      resolved: resolveExport({
        family,
        file: of.file,
        name: member,
        seen,
      },),
    },);
  }
  if (of.kind !== 'object')
    return OPAQUE;
  /**
   The name of the member as a function.
   */
  const name = `${of.name}.${member}`;
  return (family.get(of.file,)
    ?.declared
    .some(function isMember(declared,): boolean {
      return declared.name === name;
    },)
    ?? false)
    ? {
      kind: 'function',
      file: of.file,
      name,
    }
    : OPAQUE;
}

/**
 What a name bound at the top level of a module stands for, followed through imports, aliases, destructured members and
 dynamic imports.

 @param family - the modules of the sources

 @param file - path of the module

 @param name - the name

 @param seen - the bindings already followed, so a name that leads back to itself ends

 @returns What it stands for; a name the module binds nowhere is a free name

 @example
 ```ts
 const resolved = resolveName({ family, file: 'a/src/a.ts', name: 'build', seen: new Set<string>(), },);
 ```
 */
function resolveName(
  {
    family,
    file,
    name,
    seen,
  }: {
    readonly family: Family;
    readonly file: string;
    readonly name: string;
    readonly seen: ReadonlySet<string>;
  },
): Resolved {
  /**
   What the module binds the name to, absent when it binds none.
   */
  const binding = family.get(file,)
    ?.bindings
    .get(name,);
  /**
   The binding as a place to stop when the walk comes back to it.
   */
  const place = `${file}#${name}`;
  if (binding === undefined) {
    return {
      kind: 'free',
      name,
    };
  }
  if (seen.has(place,))
    return OPAQUE;
  /**
   The bindings followed, this one included.
   */
  const followed = new Set([
    ...seen,
    place,
  ],);
  if ((binding.kind === 'function') || (binding.kind === 'object')) {
    return {
      kind: binding.kind,
      file,
      name,
    };
  }
  if (binding.kind === 'path') {
    return resolvePath({
      family,
      file,
      path: binding.path,
      seen: followed,
    },);
  }
  /**
   The module the binding comes from, empty when it is outside the sources.
   */
  const target = targetOf({
    family,
    from: file,
    specifier: binding.specifier,
  },);
  if (binding.kind === 'module') {
    /**
     The module as a whole, before any member is read off it.
     */
    const whole: Resolved = (target === '')
      ? { kind: 'outside', }
      : {
        kind: 'namespace',
        file: target,
      };
    return binding.members
      .reduce(
      function readMember(
        of,
        member,
      ): Resolved {
        return memberOf({
          family,
          of,
          member,
          seen: followed,
        },);
      },
      whole,
      );
  }
  if (target === '') {
    return (binding.imported === '*')
      ? { kind: 'outside', }
      : {
        kind: 'free',
        name: (binding.imported === 'default') ? name : binding.imported,
      };
  }
  if (binding.imported === '*') {
    return {
      kind: 'namespace',
      file: target,
    };
  }
  return valueOf({
    resolved: resolveExport({
      family,
      file: target,
      name: binding.imported,
      seen: followed,
    },),
  },);
}

/**
 What a chain of names written in a module stands for.

 @param family - the modules of the sources

 @param file - path of the module that writes the chain

 @param path - the names in order, the first of which is bound at the top level of the module or free

 @param seen - the bindings already followed

 @returns What the chain stands for

 @example
 ```ts
 const resolved = resolvePath({ family, file, path: ['kit', 'build',], seen: new Set<string>(), },);
 ```
 */
export function resolvePath(
  {
    family,
    file,
    path,
    seen,
  }: {
    readonly family: Family;
    readonly file: string;
    readonly path: readonly string[];
    readonly seen: ReadonlySet<string>;
  },
): Resolved {
  /**
   The first name and the members after it.
   */
  const [head = '', ...members] = path;
  return members.reduce(
    function readMember(
      of,
      member,
    ): Resolved {
      return memberOf({
        family,
        of,
        member,
        seen,
      },);
    },
    resolveName({
      family,
      file,
      name: head,
      seen,
    },),
  );
}

/**
 What a module exports under a name, followed through the modules it exports from.

 @param family - the modules of the sources

 @param file - path of the module

 @param name - the name it exports, or `default`

 @param seen - the exports already followed, so a module that exports itself ends

 @returns What the export stands for, `none` when the module exports no such name

 @example
 ```ts
 const resolved = resolveExport({ family, file: 'a/src/index.ts', name: 'build', seen: new Set<string>(), },);
 ```
 */
function resolveExport(
  {
    family,
    file,
    name,
    seen,
  }: {
    readonly family: Family;
    readonly file: string;
    readonly name: string;
    readonly seen: ReadonlySet<string>;
  },
): Resolved {
  /**
   The export as a place to stop when the walk comes back to it.
   */
  const place = `${file}#export:${name}`;
  /**
   What the module exports under the name, absent when it exports none.
   */
  const entry = family.get(file,)
    ?.exports
    .get(name,);
  if (seen.has(place,))
    return NONE;
  /**
   The exports followed, this one included.
   */
  const followed = new Set([
    ...seen,
    place,
  ],);
  if (entry === undefined)
    return resolveStarExport({
      family,
      file,
      name,
      seen: followed,
    },);
  if (entry.kind === 'local') {
    /**
     What the local name stands for.
     */
    const local = resolveName({
      family,
      file,
      name: entry.name,
      seen: followed,
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
    : resolveExport({
      family,
      file: target,
      name: entry.imported,
      seen: followed,
    },);
}

/**
 What a module exports under a name that is none of its own entries: what the modules it exports everything of export,
 the first that exports the name winning, and never the default.

 @param family - the modules of the sources

 @param file - path of the module

 @param name - the name it exports

 @param seen - the exports already followed

 @returns What the export stands for, `none` when no module it exports everything of exports the name

 @example
 ```ts
 const resolved = resolveStarExport({ family, file, name: 'build', seen: new Set<string>(), },);
 ```
 */
function resolveStarExport(
  {
    family,
    file,
    name,
    seen,
  }: {
    readonly family: Family;
    readonly file: string;
    readonly name: string;
    readonly seen: ReadonlySet<string>;
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
      : resolveExport({
        family,
        file: target,
        name,
        seen,
      },);
    if (resolved.kind !== 'none')
      return resolved;
  }
  return NONE;
}

//endregion Which function a name stands for

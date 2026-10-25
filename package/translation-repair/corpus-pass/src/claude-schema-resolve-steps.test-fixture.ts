import type {
  Family,
  Resolved,
} from './claude-schema-resolve.test-fixture.ts';
import { targetOf, } from './claude-schema-target.test-fixture.ts';

//region The steps that read one question of the resolver
// What `claude-schema-resolve.test-fixture.ts` dispatches each question to: the reading of a member of a thing, of a name
// of a module, of a chain of names, of an export of a module, and of a name a module exports everything of. The module
// that holds the dispatcher says which function a name stands for.

/**
 The thing no function is read through.
 */
export const OPAQUE: Resolved = { kind: 'opaque', };

/**
 The members of a function that call it: `build.call(...)`, `build.apply(...)` and `build.bind(...)` are `build`.
 */
const CALL_METHODS: ReadonlySet<string> = new Set([
  'apply',
  'bind',
  'call',
],);

/**
 What a module exports under a name it does not export.
 */
export const NONE: Resolved = { kind: 'none', };

/**
 What a name that leads to an export reads as, once an export that is not there is told from a value no function is read
 through.

 @param resolved - what the export resolved to

 @returns Thing no function is read through for an export that is not there, the resolved thing otherwise

 @example
 ```ts
 const read = valueOf({ resolved: NONE, },);
 // => { kind: 'opaque' }
 ```
 */
function valueOf({ resolved, }: { readonly resolved: Resolved; },): Resolved {
  return (resolved.kind === 'none') ? OPAQUE : resolved;
}

// `memberStep`, `nameStep`, `pathStep`, `exportStep` and `starStep` call one another: a name leads to a member, a
// member to an export, an export to a name. Source order cannot put each before the functions it calls, so none calls
// another by its name: each takes `resolve`, which answers a `Query` about any of the five, and `resolveQuery`, declared
// after them, hands itself to the step it dispatches to.

/**
 A question the resolver answers: what a member of a thing, a name of a module, a chain of names, an export of a module
 or a name a module exports everything of stands for.
 */
export type Query =
  | {
    /**
     What a member read off a thing stands for.
     */
    readonly kind: 'member';

    /**
     The thing.
     */
    readonly of: Resolved;

    /**
     The member's name.
     */
    readonly member: string;
  }
    | {
    /**
     What a name bound at the top level of a module stands for.
     */
      readonly kind: 'name';

    /**
     Path of the module.
     */
      readonly file: string;

    /**
     The name.
     */
      readonly name: string;
    }
    | {
    /**
     What a chain of names written in a module stands for.
     */
      readonly kind: 'path';

    /**
     Path of the module.
     */
      readonly file: string;

    /**
     The names in order.
     */
      readonly path: readonly string[];
    }
    | {
    /**
     What a module exports under a name.
     */
      readonly kind: 'export';

    /**
     Path of the module.
     */
      readonly file: string;

    /**
     The name it exports.
     */
      readonly name: string;
    }
    | {
    /**
     What a module exports under a name through the modules it exports everything of.
     */
      readonly kind: 'star';

    /**
     Path of the module.
     */
      readonly file: string;

    /**
     The name it exports.
     */
      readonly name: string;
    };

/**
 The function that answers a question about the family, given the bindings and exports already followed.
 */
export type Resolve = (
  input: {
    readonly family: Family;
    readonly seen: ReadonlySet<string>;
    readonly query: Query;
  },
) => Resolved;

/**
 What a member read off a thing stands for.

 @param family - modules of the sources

 @param of - thing

 @param member - member's name

 @param seen - bindings and exports already followed

 @param resolve - answers the questions the member leads to

 @returns Member: `call`, `apply` and `bind` of a function or of a free name are that function or name, a member of
 an object is the function `object.member` when the module declares one, an export of a module is what the module
 exports, any member of an outside module is a free name, any other member of a free name extends it

 @example
 ```ts
 const next = memberStep({ family, of: { kind: 'free', name: 'kit', }, member: 'build', seen, resolve, },);
 // => { kind: 'free', name: 'kit.build' }
 ```
 */
export function memberStep(
  {
    family,
    of,
    member,
    seen,
    resolve,
  }: {
    readonly family: Family;
    readonly of: Resolved;
    readonly member: string;
    readonly seen: ReadonlySet<string>;
    readonly resolve: Resolve;
  },
): Resolved {
  if (CALL_METHODS.has(member,) && ((of.kind === 'function') || (of.kind === 'free')))
    return of;
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
      resolved: resolve({
        family,
        seen,
        query: {
          kind: 'export',
          file: of.file,
          name: member,
        },
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

 @param family - modules of the sources

 @param file - path of the module

 @param name - name

 @param seen - bindings already followed, so a name that leads back to itself ends

 @param resolve - answers the questions the name leads to

 @returns What it stands for; a name the module binds nowhere is a free name

 @example
 ```ts
 const resolved = nameStep({ family, file: 'a/src/a.ts', name: 'build', seen: new Set<string>(), resolve, },);
 ```
 */
export function nameStep(
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
    return resolve({
      family,
      seen: followed,
      query: {
        kind: 'path',
        file,
        path: binding.path,
      },
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
        return resolve({
          family,
          seen: followed,
          query: {
            kind: 'member',
            of,
            member,
          },
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
    resolved: resolve({
      family,
      seen: followed,
      query: {
        kind: 'export',
        file: target,
        name: binding.imported,
      },
    },),
  },);
}

/**
 What a chain of names written in a module stands for.

 @param family - modules of the sources

 @param file - path of the module that writes the chain

 @param path - names in order, the first of which is bound at the top level of the module or free

 @param seen - bindings already followed

 @param resolve - answers the questions the chain leads to

 @returns What the chain stands for

 @example
 ```ts
 const resolved = pathStep({ family, file, path: ['kit', 'build',], seen: new Set<string>(), resolve, },);
 ```
 */
export function pathStep(
  {
    family,
    file,
    path,
    seen,
    resolve,
  }: {
    readonly family: Family;
    readonly file: string;
    readonly path: readonly string[];
    readonly seen: ReadonlySet<string>;
    readonly resolve: Resolve;
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
      return resolve({
        family,
        seen,
        query: {
          kind: 'member',
          of,
          member,
        },
      },);
    },
    resolve({
      family,
      seen,
      query: {
        kind: 'name',
        file,
        name: head,
      },
    },),
  );
}

//endregion The steps that read one question of the resolver

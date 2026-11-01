import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  UNPAIRED_VIEW_INTERFACES,
  VERIFIED_UNPAIRED_VIEW_COUNT,
} from '../dist/final/node/index.mjs';

/**
 Bytes in the probe buffer, wide enough for the widest member this authority names.
 */
const PROBE_BYTE_LENGTH = 16;

/**
 Instant the probe date starts from, chosen distinct in every field it can be read by.
 
 `2000-01-02T03:04:05.006Z`, so no two field readings collide and a witness argument
 that happens to match one field cannot make a different field's write invisible.
 */
const DATE_PROBE_MILLISECONDS = 946_782_245_006;

/**
 Arguments each writing member is tried with, from a fresh receiver every time.
 
 Three distinct values rather than one, because a single argument makes a probe
 vacuous whenever it names the value the receiver already holds. Measured: the first
 version of this probe passed `1` and read `setUTCDate` as read-only, since an epoch
 date already sits on day one. Any member yielding two different states across these
 arguments provably wrote, whatever the ambient time zone.
 */
const SETTER_WITNESS_ARGUMENTS: readonly number[] = [
  2,
  5,
  9,
];

/**
 Prototype carrying the runtime surface of each declared interface.
 */
const PROTOTYPE_BY_INTERFACE: ReadonlyMap<string, object> = new Map<string, object>([
  [
    'DataView',
    DataView.prototype,
  ],
  [
    'Date',
    Date.prototype,
  ],
],);

/**
 Members an engine provides that TypeScript never declares, by interface.
 
 Absence from the authority claims a member restructures its receiver, so every
 omission has to be either true or unreachable. These are unreachable: `getYear`,
 `setYear` and `toGMTString` are Annex B members no `lib.*.d.ts` declares, so no
 member lookup can arrive at the authority holding one of these names. They are
 skipped here rather than declared, because declaring them would pin a count against
 names the rule can never consult.
 */
const UNDECLARED_RUNTIME_MEMBERS: ReadonlyMap<string, ReadonlySet<string>> = new Map([
  [
    'DataView',
    new Set<string>(),
  ],
  [
    'Date',
    new Set([
      'getYear',
      'setYear',
      'toGMTString',
    ],),
  ],
],);

/**
 Reads `[[DateValue]]` through the implementation this engine ships.
 
 The probe has to observe the slot through the intrinsic and never through whatever
 `getTime` a receiver resolves, or a member that shadowed it would be measured by
 itself. The intrinsic is taken off its own property descriptor rather than referenced
 as a method, so nothing here holds a method that could lose its receiver:
 `Reflect.apply` supplies one at the call.
 
 @param date - Receiver whose slot is read.
 
 @returns milliseconds the receiver holds.
 
 @throws Error when this engine declares no intrinsic reader at all.
 
 @example
 ```ts
 intrinsicDateTime({ date: new Date(0) });
 ```
 */
function intrinsicDateTime({ date, }: { readonly date: Date; },): number {
  /**
   Own descriptor of the intrinsic reader, holding it as data.
   */
  const descriptor = Object.getOwnPropertyDescriptor(Date.prototype, 'getTime',);
  if (descriptor === undefined) {
    throw new Error(
      'This engine declares no own Date.prototype.getTime, so the probe has no intrinsic reading of the slot every date member is measured against.',
    );
  }
  return Reflect.apply(
    descriptor.value as (this: Date) => number,
    date,
    [],
  );
}

/**
 One receiver under probe and a reading of everything a member could change about it.
 */
type ProbeSubject = {
  readonly receiver: object;
  readonly snapshot: () => string;
};

/**
 Builds a fresh receiver of one interface and a reader of its whole observable state.
 
 The snapshot covers two things a member can change and not one. The specification
 state comes first: a `DataView` writes bytes of its buffer, and a `Date` writes
 `[[DateValue]]`. Own properties come second, because a member could add, remove or
 redefine one without touching the slot at all, and a slot-only reading would call
 that preservation.
 
 The date reading goes through `Date.prototype.getTime` applied by `Reflect`, not
 through `receiver.getTime()`, so a probe of an overriding member cannot be measured
 by the member it overrides.
 
 @param ownerName - Interface being probed.
 
 @returns receiver under probe and a snapshot function over its whole state.
 
 @throws Error when asked for an interface no probe here can build.
 
 @example
 ```ts
 const { receiver, snapshot } = probeSubject({ ownerName: 'Date' });
 ```
 */
function probeSubject({
  ownerName,
}: {
  readonly ownerName: string;
},): ProbeSubject {
  /**
   Own properties and their descriptors, so a redefinition reads as a change.
   */
  function ownPropertyText({ receiver, }: { readonly receiver: object; },): string {
    return JSON.stringify(
      Reflect.ownKeys(receiver,)
        .map(function describeKey(key,): string {
          return `${String(key,)}:${
            JSON.stringify(Object.getOwnPropertyDescriptor(receiver, key,),) ?? ''
          }`;
        },),
    );
  }
  if (ownerName === 'DataView') {
    /**
     Buffer the probe view reads and writes.
     */
    const buffer = new ArrayBuffer(PROBE_BYTE_LENGTH,);
    /**
     View under probe, over a buffer this can read independently of it.
     */
    const view = new DataView(buffer,);
    return {
      receiver: view,
      snapshot: (): string =>
        `${
          [
            ...new Uint8Array(buffer,),
          ]
            .join(',',)
        }|${ownPropertyText({ receiver: view, },)}`,
    };
  }
  if (ownerName === 'Date') {
    /**
     Date under probe, at an instant distinct in every readable field.
     */
    const date = new Date(DATE_PROBE_MILLISECONDS,);
    return {
      receiver: date,
      snapshot: (): string =>
        `${
          String(
            intrinsicDateTime({ date, },),
          )
        }|${ownPropertyText({ receiver: date, },)}`,
    };
  }
  throw new Error(
    `No probe exists for ${ownerName}, so its membership would be pinned by a count and verified by nothing.`,
  );
}

/**
 Arguments one member is called with, in the domain its own interface declares.
 
 A buffer accessor takes an offset and, when it writes, a value; a date setter takes
 one field value. Both are plain numbers on purpose: a recording argument would report
 its own coercion, and what this probe asks is whether the receiver changed.
 
 @param ownerName - Interface declaring member.
 
 @param memberName - Member being exercised.
 
 @param witness - Value a writing member is asked to store.
 
 @returns argument list for the call.
 
 @example
 ```ts
 memberArguments({ ownerName: 'Date', memberName: 'setMonth', witness: 5 });
 ```
 */
function memberArguments({
  ownerName,
  memberName,
  witness,
}: {
  readonly ownerName: string;
  readonly memberName: string;
  readonly witness: number;
},): readonly unknown[] {
  if (ownerName === 'DataView') {
    if (!memberName.startsWith('set',))
      return [0,];
    /* A `Big` member rejects a number outright, so the written value has to match the
     * member's own domain or the probe fails on argument conversion rather than
     * measuring anything. */
    return memberName.includes('Big',)
      ? [0, BigInt(witness,),]
      : [0, witness,];
  }
  return memberName.startsWith('set',)
    ? [witness,]
    : [];
}

/**
 Calls one member once, reporting whether it ran to completion.
 
 A property is read rather than called, which is the whole of what reading one does.
 Completion is reported because an unchanged receiver is only evidence of preservation
 when the member actually ran: a member that throws on its arguments leaves everything
 alone and would otherwise read as proof it changes nothing.
 
 @param ownerName - Interface declaring member.
 
 @param receiver - Receiver under probe.
 
 @param memberName - Member being exercised.
 
 @param witness - Value a writing member is asked to store.
 
 @returns whether the call completed without throwing.
 
 @example
 ```ts
 exerciseMember({ ownerName: 'Date', receiver, memberName: 'getTime', witness: 2 });
 ```
 */
function exerciseMember({
  ownerName,
  receiver,
  memberName,
  witness,
}: {
  readonly ownerName: string;
  readonly receiver: object;
  readonly memberName: string;
  readonly witness: number;
},): boolean {
  try {
    /**
     Value the member holds, which for a property is the whole operation.
     */
    const held = (receiver as unknown as Record<string, unknown>)[memberName];
    if ((typeof held) !== 'function')
      return true;
    (held as (this: object, ...args: readonly unknown[]) => unknown)
      .apply(
      receiver,
      [...memberArguments({
        ownerName,
        memberName,
        witness,
      },),],
      );
    return true;
  }
  catch (error: unknown) {
    void error;
    return false;
  }
}

/**
 Runs one member against every witness argument, each from a receiver of its own.
 
 @param ownerName - Interface declaring member.
 
 @param memberName - Member being exercised.
 
 @returns whether any argument changed the receiver, and whether any call threw.
 
 @example
 ```ts
 probeMember({ ownerName: 'Date', memberName: 'setMonth' });
 ```
 */
function probeMember({
  ownerName,
  memberName,
}: {
  readonly ownerName: string;
  readonly memberName: string;
},): {
  readonly changed: boolean;
  readonly threw: boolean;
} {
  /**
   What the witnesses showed between them, accumulated across fresh receivers.
   */
  const outcome = {
    changed: false,
    threw: false,
  };
  for (const witness of SETTER_WITNESS_ARGUMENTS) {
    const { receiver, snapshot, } = probeSubject({ ownerName, },);
    /**
     State before the member runs.
     */
    const before = snapshot();
    if (!exerciseMember({
      ownerName,
      receiver,
      memberName,
      witness,
    },))
      outcome.threw = true;
    if (snapshot() !== before)
      outcome.changed = true;
  }
  return outcome;
}

await describe({
  name: 'unpaired read-only view membership',
  concurrency: 1,
  children: [
    it({
      name: 'keeps every declared member from changing the receiver, and every absent one changing it',
      fn: async () => {
        /**
         Declared members that changed the receiver despite claiming to be read-only.
         */
        const wroteDespiteDeclared: string[] = [];
        /**
         Absent members that changed nothing despite being treated as mutators.
         */
        const inertDespiteAbsent: string[] = [];
        /**
         Declared members whose evidence of preservation is a call that never ran.
         */
        const threwDespiteDeclared: string[] = [];
        for (const [ownerName, declared,] of UNPAIRED_VIEW_INTERFACES) {
          /**
           Runtime surface of the interface, as the engine actually exposes it.
           */
          const prototype = PROTOTYPE_BY_INTERFACE.get(ownerName,);
          if (prototype === undefined) {
            throw new Error(
              `No probe exists for ${ownerName}, so its membership would be pinned by a count and verified by nothing.`,
            );
          }
          /**
           Names this interface is probed on, in the engine's own reflection.
           
           `Reflect.ownKeys` rather than `Object.getOwnPropertyNames`, so a symbol
           member is excluded by a filter that says so instead of by an enumeration
           that never mentioned it. `Date.prototype[Symbol.toPrimitive]` preserves its
           receiver and is still absent from the authority, because
           `collectionStructureClaim` rejects a computed member name before consulting
           it: the name stays unrecognized rather than being claimed to mutate.
           */
          const surface = Reflect.ownKeys(prototype,)
            .filter(function isProbeableName(memberName,): memberName is string {
              return ((typeof memberName) === 'string')
                && (memberName !== 'constructor')
                && (!(UNDECLARED_RUNTIME_MEMBERS.get(ownerName,)
                  ?.has(memberName,) ?? false));
            },);
          for (const memberName of surface) {
            const { changed, threw, } = probeMember({
              ownerName,
              memberName,
            },);
            if (declared.has(memberName,) && changed)
              wroteDespiteDeclared.push(`${ownerName}.${memberName}`,);
            if (declared.has(memberName,) && threw)
              threwDespiteDeclared.push(`${ownerName}.${memberName}`,);
            if ((!declared.has(memberName,)) && (!changed))
              inertDespiteAbsent.push(`${ownerName}.${memberName}`,);
          }
        }
        /* A non-empty first list means this authority calls a member read-only that writes the
         * receiver, which would let the rule offer a parameter it rewrites. A non-empty second
         * means it calls a member mutating that changes nothing, which costs a discharge the
         * derivation should have made. The entry goes rather than either comparison. */
        expect(wroteDespiteDeclared,).toEqual([],);
        expect(inertDespiteAbsent,).toEqual([],);
        /* And a non-empty third means a declared member's clean reading came from a call that
         * threw, which is the vacuous pass this probe exists to avoid. */
        expect(threwDespiteDeclared,).toEqual([],);
      },
    },),
    it({
      name: 'skips only names the engine really provides and the declarations really omit',
      fn: async () => {
        /* A skip is a hole in the probe, so each one has to name something real. If an
         * engine drops an Annex B member the skip stops covering anything and should go,
         * and if a name is misspelled it silently covers nothing at all. */
        /**
         Skipped names absent from the interface they claim to belong to.
         */
        const skippedButAbsent: string[] = [];
        for (const [ownerName, skipped,] of UNDECLARED_RUNTIME_MEMBERS) {
          /**
           Runtime surface the skips are checked against.
           */
          const prototype = PROTOTYPE_BY_INTERFACE.get(ownerName,);
          for (const memberName of skipped) {
            if ((prototype === undefined)
              || (!Object.hasOwn(prototype, memberName,)))
              skippedButAbsent.push(`${ownerName}.${memberName}`,);
          }
        }
        expect(skippedButAbsent,).toEqual([],);
        /* And the one symbol member is present, so the filter excluding it is excluding
         * something rather than restating an empty case. */
        expect(
          Object.hasOwn(Date.prototype, Symbol.toPrimitive,),
        ).toBe(true,);
      },
    },),
    it({
      name: 'keeps the pinned entry count matching the declared membership',
      fn: async () => {
        /**
         Members declared across every unpaired interface.
         */
        const declaredCount = [...UNPAIRED_VIEW_INTERFACES.values(),]
          .reduce(function addSize(total, members,): number {
            return total + members.size;
          }, 0,);
        expect(declaredCount,).toBe(VERIFIED_UNPAIRED_VIEW_COUNT,);
      },
    },),
  ],
},);

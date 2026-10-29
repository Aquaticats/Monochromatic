/**
 Tests for the inlined function-identity copy: name, wrapped `toString`,
 own properties, parameter count, and prototype.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  mimicFunction,
  pMemoize,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'mimic-function copy',
  children: [
    it({
      name: 'preserves the original function name',
      fn: async () => {
        /**
         Named wrapped function whose identity is copied.
         */
        async function namedFixture(): Promise<number> {
          return 1;
        };
        const memoized = pMemoize({
          fn: namedFixture,
        },);

        expect(memoized.name,).toBe('namedFixture',);
      },
    },),

    it({
      name: 'reports the wrapped body through toString with a wrapper marker',
      fn: async () => {
        /**
         Wrapped function whose body text is wrapped.
         */
        async function namedFixture(): Promise<number> {
          return 1;
        };
        const memoized = pMemoize({
          fn: namedFixture,
        },);

        expect(memoized.toString(),).toBe(`/* Wrapped with memoized() */\n${namedFixture.toString()}`,);
      },
    },),

    it({
      name: 'copies own properties from the source function',
      fn: async () => {
        /**
         Wrapped function carrying one custom own property.
         */
        async function fixture(): Promise<number> {
          return 1;
        };
        Object.defineProperty(
          fixture,
          'custom',
          {
            value: 'copied',
            writable: true,
            enumerable: true,
            configurable: true,
          },
        );
        const memoized = pMemoize({
          fn: fixture,
        },);

        expect((memoized as unknown as {
          readonly custom: string;
        }).custom,).toBe('copied',);
      },
    },),

    it({
      name: 'keeps its own parameter count instead of copying the source one',
      fn: async () => {
        /**
         Wrapped function with more parameters than the memoized wrapper.
         */
        async function fixture(
          first: string,
          second: string,
        ): Promise<string> {
          return `${first}-${second}`;
        };
        const memoized = pMemoize({
          fn: fixture,
        },);

        expect(memoized.length,).toBe(1,);
        expect(fixture.length,).toBe(2,);
      },
    },),

    it({
      name: 'adopts the source prototype chain',
      fn: async () => {
        /**
         Wrapped async function whose prototype the wrapper adopts.
         */
        async function fixture(): Promise<number> {
          return 1;
        };
        const memoized = pMemoize({
          fn: fixture,
        },);

        expect(Reflect.getPrototypeOf(memoized,),).toBe(Reflect.getPrototypeOf(fixture,),);
      },
    },),

    it({
      name: 'copies a non-configurable source property and keeps its descriptor',
      fn: async () => {
        /**
         Wrapped function carrying one non-configurable own property.
         */
        async function fixture(): Promise<number> {
          return 1;
        };
        Object.defineProperty(
          fixture,
          'locked',
          {
            value: 'source',
            writable: false,
            enumerable: false,
            configurable: false,
          },
        );
        const memoized = pMemoize({
          fn: fixture,
        },);

        expect((memoized as unknown as {
          readonly locked: string;
        }).locked,).toBe('source',);
        /**
         Own descriptor the copy installed on the wrapper.
         */
        const copiedDescriptor = Object.getOwnPropertyDescriptor(
          memoized,
          'locked',
        ) as PropertyDescriptor;
        expect(copiedDescriptor.configurable,).toBe(false,);
        expect(copiedDescriptor.writable,).toBe(false,);
      },
    },),
  ],
},);

/**
 One hand-crafted descriptor-pair case for the copy-legality behavior.
 */
type CopyCase = {
  /**
   Case description shown as the test name.
   */
  readonly name: string;
  /**
   Whether the target already owns the property under test.
   */
  readonly hasTargetProperty: boolean;
  /**
   Target's existing property descriptor, seeded when present.
   */
  readonly targetDescriptor: PropertyDescriptor;
  /**
   Source's property descriptor to copy.
   */
  readonly sourceDescriptor: PropertyDescriptor;
  /**
   Whether the copy must reach `Object.defineProperty`.
   */
  readonly expectCopy: boolean;
};

/**
 Descriptor-pair cases covering every branch of the copy-legality
 predicate: missing and configurable targets always copy, non-configurable
targets copy only when every flag and the value match or the value changes
through `writable`, and conflicts are skipped under `ignoreNonConfigurable`.
 */
const COPY_CASES: readonly CopyCase[] = [
  {
    name: 'copies onto a missing target property',
    hasTargetProperty: false,
    targetDescriptor: {
      value: 'target',
      writable: true,
      enumerable: true,
      configurable: true,
    },
    sourceDescriptor: {
      value: 'source',
      writable: true,
      enumerable: true,
      configurable: true,
    },
    expectCopy: true,
  },
  {
    name: 'copies onto a configurable target property',
    hasTargetProperty: true,
    targetDescriptor: {
      value: 'target',
      writable: false,
      enumerable: false,
      configurable: true,
    },
    sourceDescriptor: {
      value: 'source',
      writable: true,
      enumerable: true,
      configurable: true,
    },
    expectCopy: true,
  },
  {
    name: 'copies non-configurable properties whose flags and value match',
    hasTargetProperty: true,
    targetDescriptor: {
      value: 'same',
      writable: false,
      enumerable: false,
      configurable: false,
    },
    sourceDescriptor: {
      value: 'same',
      writable: false,
      enumerable: false,
      configurable: false,
    },
    expectCopy: true,
  },
  {
    name: 'copies value changes through writable non-configurable properties',
    hasTargetProperty: true,
    targetDescriptor: {
      value: 'target',
      writable: true,
      enumerable: false,
      configurable: false,
    },
    sourceDescriptor: {
      value: 'source',
      writable: true,
      enumerable: false,
      configurable: false,
    },
    expectCopy: true,
  },
  {
    name: 'skips non-configurable properties whose value differs',
    hasTargetProperty: true,
    targetDescriptor: {
      value: 'target',
      writable: false,
      enumerable: false,
      configurable: false,
    },
    sourceDescriptor: {
      value: 'source',
      writable: false,
      enumerable: false,
      configurable: false,
    },
    expectCopy: false,
  },
  {
    name: 'skips non-configurable properties whose writable flag differs',
    hasTargetProperty: true,
    targetDescriptor: {
      value: 'same',
      writable: false,
      enumerable: false,
      configurable: false,
    },
    sourceDescriptor: {
      value: 'same',
      writable: true,
      enumerable: false,
      configurable: false,
    },
    expectCopy: false,
  },
  {
    name: 'skips non-configurable properties whose enumerable flag differs',
    hasTargetProperty: true,
    targetDescriptor: {
      value: 'same',
      writable: false,
      enumerable: false,
      configurable: false,
    },
    sourceDescriptor: {
      value: 'same',
      writable: false,
      enumerable: true,
      configurable: false,
    },
    expectCopy: false,
  },
  {
    name: 'skips non-configurable properties whose configurable flag differs',
    hasTargetProperty: true,
    targetDescriptor: {
      value: 'same',
      writable: false,
      enumerable: false,
      configurable: false,
    },
    sourceDescriptor: {
      value: 'same',
      writable: false,
      enumerable: false,
      configurable: true,
    },
    expectCopy: false,
  },
];

await describe({
  name: mimicFunction.name,
  children: [
    ...COPY_CASES.map(function describeCopyCase(copyCase: CopyCase,): ReturnType<typeof it> {
      return it({
        name: copyCase.name,
        fn: async () => {
          /**
           Source function carrying the case's property descriptor.
           */
          function source(): void {
            void 0;
          }
          Object.defineProperty(
            source,
            'locked',
            copyCase.sourceDescriptor,
          );
          /**
           Target function carrying the case's existing descriptor.
           */
          function target(): void {
            void 0;
          }

          if (copyCase.hasTargetProperty)
            Object.defineProperty(
              target,
              'locked',
              copyCase.targetDescriptor,
            );

          /**
           Property names the copy attempted to define.
           */
          const defined: (string | symbol)[] = [];
          /**
           Target proxy recording every attempted property definition.
           */
          const recordingTarget = new Proxy(target, {
            defineProperty: function recordDefine(
              proxyTarget: typeof target,
              property: string | symbol,
              descriptor: PropertyDescriptor,
            ): boolean {
              defined.push(property,);
              return Reflect.defineProperty(
                proxyTarget,
                property,
                descriptor,
              );
            },
          },);

          /**
           Whether the mimic call failed; the legal path never throws.
           */
          let threw = false;
          /**
           Failure thrown by an illegal copy attempt, when one happened.
           */
          let failure: unknown;
          try {
            mimicFunction({
              to: recordingTarget,
              from: source,
              ignoreNonConfigurable: true,
            },);
          }
          catch (error) {
            threw = true;
            failure = error;
          }

          expect({
            copied: defined.includes('locked',),
            threw,
          },).toEqual({
            copied: copyCase.expectCopy,
            threw: false,
          },);
          if (threw)
            expect(failure,).toBeInstanceOf(TypeError,);
        },
      },);
    },),

    it({
      name: 'throws on non-configurable conflicts unless ignoreNonConfigurable is set',
      fn: async () => {
        /**
         Source function whose `locked` property conflicts with the target's.
         */
        function source(): void {
          void 0;
        }
        Object.defineProperty(
          source,
          'locked',
          {
            value: 'source',
            writable: false,
            enumerable: false,
            configurable: false,
          },
        );
        /**
         Target function whose non-configurable `locked` property blocks the
         copy.
         */
        function target(): void {
          void 0;
        }
        Object.defineProperty(
          target,
          'locked',
          {
            value: 'target',
            writable: false,
            enumerable: false,
            configurable: false,
          },
        );

        let caught: unknown;
        try {
          mimicFunction({
            to: target,
            from: source,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(TypeError,);
      },
    },),

    it({
      name: 'never copies arguments or caller properties, matching upstream',
      fn: async () => {
        /**
         Source function carrying `arguments` and `caller` own properties,
         as some React Native runtimes report through `Reflect.ownKeys`.
         */
        function source(): void {
          void 0;
        }
        Object.defineProperty(
          source,
          'arguments',
          {
            value: 'copied',
            writable: true,
            enumerable: true,
            configurable: true,
          },
        );
        Object.defineProperty(
          source,
          'caller',
          {
            value: 'copied',
            writable: true,
            enumerable: true,
            configurable: true,
          },
        );
        /**
         Target function that must stay free of both properties.
         */
        function target(): void {
          void 0;
        }
        mimicFunction({
          to: target,
          from: source,
          ignoreNonConfigurable: true,
        },);
        expect(Object.getOwnPropertyDescriptor(
          target,
          'arguments',
        ),).toBeUndefined();
        expect(Object.getOwnPropertyDescriptor(
          target,
          'caller',
        ),).toBeUndefined();
      },
    },),

    it({
      name: 'keeps its own prototype property while adopting the source prototype chain',
      fn: async () => {
        /**
         Source function whose prototype must not become own on the target.
         */
        function source(): void {
          void 0;
        }
        /**
         Target function keeping its own prototype property.
         */
        function target(): void {
          void 0;
        }
        mimicFunction({
          to: target,
          from: source,
          ignoreNonConfigurable: false,
        },);
        expect(target.prototype,).not.toBe(source.prototype,);
      },
    },),

    it({
      name: 'formats the toString marker with the trimmed target name',
      fn: async () => {
        /**
         Target carrying a padded name, standing in for reflection-visible
         wrapper names.
         */
        const paddedTarget = {
          name: ' padded ',
        };
        /**
         Source function whose body text is wrapped.
         */
        function source(): number {
          return 1;
        }
        mimicFunction({
          to: paddedTarget,
          from: source,
        },);
        /**
         Wrapped `toString` the mimic installed on the padded target.
         */
        const paddedToString = Object.getOwnPropertyDescriptor(
          paddedTarget,
          'toString',
        ) as PropertyDescriptor;
        expect((paddedToString.value as () => string)(),).toBe(`/* Wrapped with padded() */\n${source.toString()}`,);
      },
    },),

    it({
      name: 'drops the marker name segment for an empty target name',
      fn: async () => {
        /**
         Target whose name is empty, so the marker omits the name segment.
         */
        const emptyTarget = {
          name: '',
        };
        /**
         Source function whose body text is wrapped.
         */
        function source(): number {
          return 1;
        }
        mimicFunction({
          to: emptyTarget,
          from: source,
        },);
        /**
         Wrapped `toString` the mimic installed on the empty-name target.
         */
        const emptyToString = Object.getOwnPropertyDescriptor(
          emptyTarget,
          'toString',
        ) as PropertyDescriptor;
        expect((emptyToString.value as () => string)(),).toBe(`/* Wrapped */\n${source.toString()}`,);
      },
    },),

    it({
      name: 'names the installed toString wrapper toString',
      fn: async () => {
        /**
         Target function receiving the wrapped `toString`.
         */
        function target(): void {
          void 0;
        }
        /**
         Source function whose body text is wrapped.
         */
        function source(): number {
          return 1;
        }
        mimicFunction({
          to: target,
          from: source,
        },);
        expect(target.toString.name,).toBe('toString',);
      },
    },),
  ],
},);

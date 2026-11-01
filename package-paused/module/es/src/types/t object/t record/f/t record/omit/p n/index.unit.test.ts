import { types, } from '@monochromatic-dev/module-es';
import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

const { $, } = types.object.record.from.record.omit.named;

await describe({
  name: $.name,
  children: [
    it({
      name: 'omits specified keys from object',
      fn: async () => {
        const user = { id: 1, name: 'Alice', password: 'secret', };
        const result = $({ original: user, toOmit: new Set(['password',] as const,), },);

        expect(result,).toEqual({ id: 1, name: 'Alice', },);
      },
    },),

    it({
      name: 'returns full object when omitting no keys',
      fn: async () => {
        const user = { id: 1, name: 'Alice', };
        const result = $({ original: user, toOmit: new Set<never>(), },);

        expect(result,).toEqual({ id: 1, name: 'Alice', },);
      },
    },),

    it({
      name: 'returns empty object when omitting all keys',
      fn: async () => {
        const user = { id: 1, name: 'Alice', };
        const result = $({ original: user,
          toOmit: new Set(['id', 'name',] as const,), },);

        expect(result,).toEqual({},);
      },
    },),

    it({
      name: 'handles symbol keys',
      fn: async () => {
        const symKey = Symbol('secret',);
        const obj = { id: 1, [symKey]: 'hidden', };
        const result = $({ original: obj, toOmit: new Set([symKey,] as const,), },);

        expect(result,).toEqual({ id: 1, },);
        expect(Object.getOwnPropertySymbols(result,),).toEqual([],);
      },
    },),

    it({
      name: 'handles numeric keys',
      fn: async () => {
        const obj = { 0: 'zero', 1: 'one', 2: 'two', };
        const result = $({ original: obj, toOmit: new Set([1,] as const,), },);

        expect(result,).toEqual({ 0: 'zero', 2: 'two', },);
      },
    },),

    it({
      name: 'throws when omitting non-existent key',
      fn: async () => {
        const user = { id: 1, name: 'Alice', };
        expect(() =>
          $({
            original: user,
            // @ts-expect-error; intentionally passing non-existent key to test runtime error
            toOmit: new Set(['nonexistent',] as const,),
          },)
        )
          .toThrow('Key not found in iterable: nonexistent',);
      },
    },),

    it({
      name: 'preserves non-omitted value types in result',
      fn: async () => {
        const original = { id: 1, name: 'Alice', active: true, } as const;
        const result = $({ original, toOmit: new Set(['active',] as const,), },);

        expect(result,).toHaveProperty('id',);
        expect(result,).toHaveProperty('name',);
        expectTypeOf(result,).toEqualTypeOf<Omit<typeof original, 'active'>>();
        expectTypeOf(result.id,).toEqualTypeOf<1>();
        expectTypeOf(result.name,).toEqualTypeOf<'Alice'>();
      },
    },),

    it({
      name: 'type narrows correctly to Omit type',
      fn: async () => {
        const original = { a: 1, b: 'two', c: true, };
        const result = $({ original, toOmit: new Set(['b',] as const,), },);

        expect(result,).toHaveProperty('a',);
        expect(result,).toHaveProperty('c',);
        expect(result,).not.toHaveProperty('b',);
      },
    },),

    it({
      name: 'handles mixed key types',
      fn: async () => {
        const symKey = Symbol('sym',);
        const obj = { str: 'string', 42: 'numeric', [symKey]: 'symbol', other: 'keep', };
        const result = $({ original: obj,
          toOmit: new Set(['str', 42, symKey,] as const,), },);

        expect(result,).toEqual({ other: 'keep', },);
        expect(Object.getOwnPropertySymbols(result,),).toEqual([],);
      },
    },),
  ],
},);

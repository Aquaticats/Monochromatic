import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  _jsValueToTomlText,
  emptyTomlEdit,
  parseTomlEdit,
  tomlFloat,
  tomlGetValue,
  tomlInteger,
  tomlSet,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core';

/**
 Canonical options used when probing the built scalar encoder directly.
 */
const OPTIONS = emptyTomlEdit().canonical;

await describe({
  name: 'tagged and styled value encoding',
  children: [
    it({
      name: 'spells integer and every nonfinite float explicitly',
      fn: async () => {
        expect(_jsValueToTomlText({ input: tomlInteger(42n,), options: OPTIONS, },),).toBe('42',);
        expect(_jsValueToTomlText({ input: tomlFloat(1.5,), options: OPTIONS, },),).toBe('1.5',);
        expect(_jsValueToTomlText({ input: tomlFloat(1e21,), options: OPTIONS, },),).toBe('1e+21',);
        expect(_jsValueToTomlText({ input: tomlFloat(Number.NaN,), options: OPTIONS, },),).toBe('nan',);
        expect(_jsValueToTomlText({ input: tomlFloat(Infinity,), options: OPTIONS, },),).toBe('inf',);
        expect(_jsValueToTomlText({ input: tomlFloat(-Infinity,), options: OPTIONS, },),).toBe('-inf',);
      },
    },),
    it({
      name: 'treats a non-string tomlKind as an ordinary inline table',
      fn: async () => {
        expect(_jsValueToTomlText({
          input: { tomlKind: 42, value: 1, },
          options: OPTIONS,
        },),).toBe('{ tomlKind = 42, value = 1, }',);
      },
    },),
    it({
      name: 'accepts a plain object with a null prototype',
      fn: async () => {
        const input = { __proto__: null, x: 1, };
        expect(Object.getPrototypeOf(input,),).toBe(null,);
        expect(_jsValueToTomlText({ input, options: OPTIONS, },),).toBe('{ x = 1, }',);
      },
    },),
    it({
      name: 'retains multiline literal spelling on an equal string re-set',
      fn: async () => {
        const source = "v = '''\nhello\nworld'''\n";
        const edit = parseTomlEdit({ source, },);
        const value = tomlGetValue({ edit, path: ['v',], },);
        const updated = tomlSet({ edit, path: ['v',], value, },);
        expect(tomlStringify({ edit: updated, },),).toBe(source,);
      },
    },),
  ],
},);

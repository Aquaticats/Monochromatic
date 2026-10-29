/**
 Property-based fuzz tests for the TOML wrappers in `./toml.ts`.
 
 The wrappers bottom out in `@monochromatic-dev/toml-edit-core`, which
 this monorepo owns; the goal here is to prove file-enforcer never exposes
 unexpected or undocumented behavior on top of it. Properties:
 `editTomlKey` and `getTomlProperty` are total over arbitrary input,
 succeeding or throwing only the declared `TomlEditError` (never an
 undeclared crash); a string value written at a bare key reads back
 byte-identically (a round-trip across the TOML string-escaping boundary);
 and re-applying the same edit is idempotent.
 
 Run plan and seed policy: see `../fuzz-budget.ts`.
 
 @module
 */

import {
  array,
  assert,
  asyncProperty,
  constantFrom,
  jsonValue,
  nat,
  oneof,
  record,
  string,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { TomlEditError, } from '@monochromatic-dev/toml-edit-core/ts';

import { fuzzRunPlan, } from '../fuzz-budget.ts';
import {
  editTomlKey,
  getTomlProperty,
} from './toml.ts';

//region Constants and arbitraries

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 Arbitrary structured path of string keys and numeric indices.
 */
const pathArbitrary = array(
  oneof(string(), nat(),),
  {
    minLength: 1,
    maxLength: 4,
  },
);

/**
 Arbitrary value to write; spans the JSON-ish shapes the wrappers accept.
 */
const valueArbitrary = jsonValue();

/**
 Arbitrary TOML source: mostly invalid text unioned with small valid
 documents, so both the parse-error and edit paths are exercised.
 */
const contentArbitrary = oneof(
  string(),
  constantFrom(
    '',
    'a = 1\n',
    '[table]\nkey = "value"\n',
    '# comment\nname = "x"\n',
    '[[items]]\nid = 1\n',
  ),
);

/**
 Arbitrary single TOML bare key (letters, digits, underscore, hyphen).
 */
const bareKeyArbitrary = string({
  minLength: 1,
  maxLength: 12,
  unit: constantFrom(
    'a',
    'b',
    'Z',
    '0',
    '9',
    '_',
    '-',
  ),
},);

//endregion Constants and arbitraries

await describe({
  name: '',
  children: [
    //region editTomlKey

    describe({
      name: editTomlKey.name,
      children: [
        it({
          name: 'returns a string or throws only TomlEditError for arbitrary input',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                record({
                  content: contentArbitrary,
                  path: pathArbitrary,
                  value: valueArbitrary,
                },),
                async function editTotality({
                  content,
                  path,
                  value,
                },) {
                  try {
                    /**
                     Edited TOML text.
                     */
                    const edited = editTomlKey({
                      content,
                      path,
                      value,
                    },);
                    expect(typeof edited,).toBe('string',);
                  }
                  catch (caught: unknown) {
                    expect(caught,).toBeInstanceOf(TomlEditError,);
                  }
                },
              ),
              RUN.params,
            );
          },
        },),

        it({
          name: 're-applying the same edit is idempotent',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                record({
                  key: bareKeyArbitrary,
                  value: string(),
                },),
                async function idempotent({
                  key,
                  value,
                },) {
                  /**
                   Document after the first edit of an empty source.
                   */
                  const once = editTomlKey({
                    content: '',
                    path: [key,],
                    value,
                  },);
                  /**
                   Document after repeating the identical edit.
                   */
                  const twice = editTomlKey({
                    content: once,
                    path: [key,],
                    value,
                  },);
                  expect(twice,).toBe(once,);
                },
              ),
              RUN.params,
            );
          },
        },),
      ],
    },),

    //endregion editTomlKey

    //region getTomlProperty

    describe({
      name: getTomlProperty.name,
      children: [
        it({
          name: 'is total over arbitrary input, throwing only TomlEditError',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                record({
                  content: contentArbitrary,
                  path: pathArbitrary,
                },),
                async function getTotality({
                  content,
                  path,
                },) {
                  try {
                    /**
                     Extracted value, or undefined when missing.
                     */
                    const value: unknown = getTomlProperty({
                      content,
                      path,
                    },);
                    expect(((typeof value) === 'string') || (value === undefined),).toBe(true,);
                  }
                  catch (caught: unknown) {
                    expect(caught,).toBeInstanceOf(TomlEditError,);
                  }
                },
              ),
              RUN.params,
            );
          },
        },),

        it({
          name: 'reads back a string value written at a bare key',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                record({
                  key: bareKeyArbitrary,
                  value: string(),
                },),
                async function roundTrips({
                  key,
                  value,
                },) {
                  /**
                   Document holding the written value at the bare key.
                   */
                  const content = editTomlKey({
                    content: '',
                    path: [key,],
                    value,
                  },);
                  expect(
                    getTomlProperty({
                      content,
                      path: [key,],
                    },),
                  ).toBe(value,);
                },
              ),
              RUN.params,
            );
          },
        },),
      ],
    },),

    //endregion getTomlProperty
  ],
},);

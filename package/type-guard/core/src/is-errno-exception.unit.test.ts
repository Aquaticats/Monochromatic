/**
 `isErrnoException` accepts only an error carrying an errno code.
 
 `package/wg/cli-quicker` settled the body this package takes over: Node's
 filesystem and process functions throw real `Error` instances whose `code`
 names the condition, so a guard that reads the code must accept those and
 refuse anything else. A bare object shaped like an errno comes from some other
 thrower, and reading its `code` as a missing file would swallow it. The suite
 pins that whole decision, so a later "simplification" to `'code' in value`
 fails here instead of making a thrower's value look like a missing path.
 */
import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isErrnoException, } from '../dist/final/neutral/index.mjs';

//region Fixtures

/**
 Code Node reports for a path that does not exist.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 Package error carrying an errno code, standing in for a thrower that attaches
 a code to an error of its own.
 */
class SamplePackageError extends Error {
  /** Name the class carries into a stack trace. */
  override readonly name: string = 'SamplePackageError';
}

/**
 Error carrying an errno code, as Node's filesystem functions throw.
 
 @returns Error whose `code` names a missing path.
 */
function errnoError(): NodeJS.ErrnoException {
  /**
   The error as Node throws it.
   */
  const error: NodeJS.ErrnoException = new Error(`open '/missing': no such file or directory`,);
  error.code = MISSING_PATH_CODE;
  return error;
}

//endregion Fixtures

await describe({
  name: isErrnoException.name,
  children: [
    //region Accepted values

    it({
      name: 'accepts the error Node throws for a path it cannot open',
      fn: async () => {
        /**
         Verdict on the error Node throws.
         */
        const verdict = isErrnoException(errnoError(),);
        expect(verdict,).toBe(true,);
      },
    },),

    it({
      name: 'accepts a package error once it carries a code',
      fn: async () => {
        /**
         Package error with an errno code attached.
         */
        const error: NodeJS.ErrnoException = new SamplePackageError('config read failed',);
        error.code = MISSING_PATH_CODE;
        expect(isErrnoException(error,),).toBe(true,);
      },
    },),

    //endregion Accepted values

    //region Rejected values

    it({
      name: 'rejects an error with no code, which names no errno condition',
      fn: async () => {
        /**
         Verdict on an error with no code.
         */
        const verdict = isErrnoException(new Error('plain failure',),);
        expect(verdict,).toBe(false,);
      },
    },),

    it({
      name: 'rejects an errno-shaped value that is not an error',
      fn: async () => {
        expect(isErrnoException({ code: MISSING_PATH_CODE, },),).toBe(false,);
      },
    },),

    //endregion Rejected values

    //region Primitive values

    ...([
      { name: 'undefined', value: undefined, },
      { name: 'null', value: null, },
      { name: 'a string', value: MISSING_PATH_CODE, },
      { name: 'a number', value: 0, },
      { name: 'a boolean', value: true, },
    ] as const).map(function primitiveCase(fixture,) {
      return it({
        name: `rejects ${fixture.name}, which carries no code at all`,
        fn: async () => {
          expect(isErrnoException(fixture.value,),).toBe(false,);
        },
      },);
    },),

    //endregion Primitive values

    //region Narrowing

    it({
      name: 'narrows an unknown caught value so its code is readable',
      fn: async () => {
        /**
         Value as a caller holds it in a `catch` clause.
         */
        const value: unknown = errnoError();
        if (!isErrnoException(value,)) {
          throw new Error('the fixture is an errno exception',);
        }
        expect(value.code,).toBe(MISSING_PATH_CODE,);
        expectTypeOf(value,).toEqualTypeOf<NodeJS.ErrnoException>();
      },
    },),

    //endregion Narrowing
  ],
},);

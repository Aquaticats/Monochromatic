/**
 One body decides whether a caught value is an errno exception.
 
 Node's filesystem and process functions throw real `Error` instances carrying
 a `code`, so a guard that reads the code must accept those and reject anything
 else: a bare object shaped like an errno, an `Error` with no code, and every
 non-object. A permissive body would read a value the thrower never built as a
 missing file, and the caller would swallow it instead of reporting it.
 */
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ConfigError,
  isErrnoException,
} from '../dist/final/node/errors.mjs';

//region Fixtures

/**
 Code Node reports for a path that does not exist.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 Error carrying an errno code, as Node's filesystem functions throw.
 
 @returns Error whose `code` names a missing path.
 */
function errnoError(): NodeJS.ErrnoException {
  /**
   The error as Node throws it.
   */
  const error: NodeJS.ErrnoException = new Error(
    `${MISSING_PATH_CODE}: no such file or directory, open '/run/wg-quicker/state.json'`,
  );
  error.code = MISSING_PATH_CODE;
  return error;
}

//endregion Fixtures

await describe({
  name: 'isErrnoException',
  children: [
    it({
      name: 'accepts the error Node throws for a path it cannot open',
      fn: async () => {
        expect(isErrnoException(errnoError(),),).toBe(true,);
      },
    },),
    it({
      name: 'accepts a package error once it carries a code, so one body covers both throwers',
      fn: async () => {
        /**
         Package error with an errno code attached.
         */
        const error: NodeJS.ErrnoException = new ConfigError('config read failed',);
        error.code = MISSING_PATH_CODE;
        expect(isErrnoException(error,),).toBe(true,);
      },
    },),
    it({
      name: 'rejects an error with no code, which names no errno condition',
      fn: async () => {
        expect(isErrnoException(new Error('plain failure',),),).toBe(false,);
      },
    },),
    it({
      name: 'rejects an errno-shaped value that is not an error',
      fn: async () => {
        expect(isErrnoException({ code: MISSING_PATH_CODE, },),).toBe(false,);
      },
    },),
    it({
      name: 'rejects values that are not objects at all',
      fn: async () => {
        /**
         Thrown values a guard may be handed.
         */
        const thrown: readonly unknown[] = [
          undefined,
          null,
          MISSING_PATH_CODE,
          0,
          true,
        ];
        for (const value of thrown) {
          expect(
            {
              value,
              errno: isErrnoException(value,),
            },
          ).toEqual(
            {
              value,
              errno: false,
            },
          );
        }
      },
    },),
  ],
},);

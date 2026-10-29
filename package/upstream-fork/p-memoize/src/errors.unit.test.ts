/**
 Tests for the error classes.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CacheDisabledError,
  NonMethodDecorationError,
  NotMemoizedError,
  PropertyDescriptorMissingError,
  PrivateMethodDecorationError,
  UnclearableCacheError,
} from '../dist/final/neutral/index.mjs';

/**
 One error class under test: how to construct it and which upstream message
 text it must carry.
 */
type ErrorSpec = {
  /**
   Builds one instance of the error class under test.
   */
  readonly construct: () => TypeError;
  /**
   Expected `name` for readable stacks.
   */
  readonly name: string;
  /**
   Expected message text, verbatim from upstream `p-memoize` where one
   exists.
   */
  readonly message: string;
};

/**
 Every error class with its expected identity and message text, generated
 into one test group each.
 */
const ERROR_SPECS: readonly ErrorSpec[] = [
  {
    construct: function constructNotMemoized(): TypeError {
      return new NotMemoizedError();
    },
    name: 'NotMemoizedError',
    message: 'Can\'t clear a function that was not memoized!',
  },
  {
    construct: function constructCacheDisabled(): TypeError {
      return new CacheDisabledError();
    },
    name: 'CacheDisabledError',
    message: 'Can\'t clear a function that doesn\'t use a cache!',
  },
  {
    construct: function constructUnclearableCache(): TypeError {
      return new UnclearableCacheError();
    },
    name: 'UnclearableCacheError',
    message: 'The cache can\'t be cleared!',
  },
  {
    construct: function constructNonMethodDecoration(): TypeError {
      return new NonMethodDecorationError();
    },
    name: 'NonMethodDecorationError',
    message: 'pMemoizeDecorator can only decorate methods',
  },
  {
    construct: function constructPrivateMethodDecoration(): TypeError {
      return new PrivateMethodDecorationError();
    },
    name: 'PrivateMethodDecorationError',
    message: 'pMemoizeDecorator cannot decorate private methods',
  },
  {
    construct: function constructPropertyDescriptorMissing(): TypeError {
      return new PropertyDescriptorMissingError();
    },
    name: 'PropertyDescriptorMissingError',
    message: 'Expected an own property descriptor for a key from Reflect.ownKeys',
  },
];

await describe({
  name: '',
  children: ERROR_SPECS.map(function describeErrorClass(spec: ErrorSpec,): ReturnType<typeof describe> {
    return describe({
      name: spec.name,
      children: [
        it({
          name: 'extends TypeError so existing TypeError handlers keep working',
          fn: async () => {
            const error = spec.construct();
            expect(error,).toBeInstanceOf(TypeError,);
            expect(error,).toBeInstanceOf(Error,);
          },
        },),

        it({
          name: 'carries the expected message text',
          fn: async () => {
            const error = spec.construct();
            expect(error.message,).toBe(spec.message,);
          },
        },),

        it({
          name: 'names itself for readable stacks',
          fn: async () => {
            const error = spec.construct();
            expect(error.name,).toBe(spec.name,);
          },
        },),
      ],
    },);
  },),
},);

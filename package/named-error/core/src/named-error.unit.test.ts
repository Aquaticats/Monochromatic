/**
 Tests for the base class that owns an error's name.

 Every case here asks what a `catch` binding sees: the `name` string, the
 prototype chain, and the message and options a caller passed. Instances come
 from the built bundle, so a rename introduced while bundling fails here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NamedError, } from '../dist/final/neutral/index.mjs';

/**
 Error carrying nothing but a message, the shape most sites take.
 */
class TabbyMissingError extends NamedError {}

/**
 Second leaf error, so a passing case cannot come from one hard-coded name.
 */
class LedgerUnreadableError extends NamedError {}

/**
 Error adding a field of its own beside the message.
 */
class QuotaExceededError extends NamedError {
  /**
   Value the refusal is about.
   */
  readonly usedBytes: number;

  /**
   @param usedBytes - stored so a caller can report the amount it exceeded
   */
  constructor(usedBytes: number,) {
    super(`quota exceeded by ${String(usedBytes,)} bytes`,);
    this.usedBytes = usedBytes;
  }
}

/**
 Error built on another named error, to prove a subclass names itself.
 */
class SpecialisedTabbyMissingError extends TabbyMissingError {}

await describe({
  name: '',
  children: [
    describe({
      name: NamedError.name,
      children: [
        it({
          name: 'names a subclass after its own class, so the subclass declares no constructor',
          fn: async () => {
            expect(new TabbyMissingError('no tabby',).name,).toBe(TabbyMissingError.name,);
            expect(new LedgerUnreadableError('ledger unreadable',).name,)
              .toBe(LedgerUnreadableError.name,);
          },
        },),

        it({
          name: 'names a further subclass after the leaf class, not the intermediate one',
          fn: async () => {
            expect(new SpecialisedTabbyMissingError('no tabby',).name,)
              .toBe(SpecialisedTabbyMissingError.name,);
          },
        },),

        it({
          name: 'names a subclass that carries fields of its own',
          fn: async () => {
            const error = new QuotaExceededError(12,);
            expect(error.name,).toBe(QuotaExceededError.name,);
            expect(error.usedBytes,).toBe(12,);
          },
        },),

        it({
          name: 'forwards the message unchanged',
          fn: async () => {
            expect(new TabbyMissingError('no tabby in the basket',).message,)
              .toBe('no tabby in the basket',);
          },
        },),

        it({
          name: 'forwards options, so a cause a caller chains survives',
          fn: async () => {
            const cause = new RangeError('basket index out of range',);
            const error = new TabbyMissingError('no tabby', { cause, },);
            expect(error.cause,).toBe(cause,);
          },
        },),

        it({
          name: 'stays an Error and a NamedError, so existing instanceof checks keep their answer',
          fn: async () => {
            const error = new TabbyMissingError('no tabby',);
            expect(error,).toBeInstanceOf(NamedError,);
            expect(error,).toBeInstanceOf(Error,);
            expect(error,).not.toBeInstanceOf(RangeError,);
          },
        },),
      ],
    },),
  ],
},);

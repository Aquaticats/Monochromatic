import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  _isQuotaExceededError as isQuotaExceededError,
} from '@monochromatic-dev/logger-core';

await describe({
  name: isQuotaExceededError.name,
  children: [
    it({
      name: 'recognizes the standard DOMException name',
      fn: async () => {
        /**
         Overflow shaped exactly as current engines raise it.
         */
        const overflow = new DOMException('full', 'QuotaExceededError',);
        expect(isQuotaExceededError(overflow,),)
          .toBe(true,);
      },
    },),

    it({
      name: 'recognizes the legacy Firefox name on any object shape',
      fn: async () => {
        expect(isQuotaExceededError({ name: 'NS_ERROR_DOM_QUOTA_REACHED', },),)
          .toBe(true,);
      },
    },),

    it({
      name: 'rejects a plain Error whose message merely mentions quota',
      fn: async () => {
        /**
         Non-quota failure that only talks about quota in prose.
         */
        const impostor = new Error('quota exceeded',);
        expect(isQuotaExceededError(impostor,),)
          .toBe(false,);
      },
    },),

    it({
      name: 'rejects non-object caught values',
      fn: async () => {
        expect(isQuotaExceededError(null,),)
          .toBe(false,);
        expect(isQuotaExceededError('QuotaExceededError',),)
          .toBe(false,);
      },
    },),
  ],
},);

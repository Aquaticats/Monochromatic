import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  clampedSum,
  describeSign,
} from './calc.ts';

await describe({
  name: '',
  children: [
    describe({
      name: clampedSum.name,
      children: [
        it({
          name: 'adds below the maximum',
          fn: async () => {
            expect(clampedSum({
              a: 1,
              b: 2,
              max: 10,
            },),).toBe(3,);
          },
        },),
        it({
          name: 'clamps at the maximum',
          fn: async () => {
            expect(clampedSum({
              a: 3,
              b: 3,
              max: 4,
            },),).toBe(4,);
          },
        },),
      ],
    },),
    describe({
      name: describeSign.name,
      children: [
        it({
          name: 'labels negatives and positives (zero branch untested on purpose)',
          fn: async () => {
            expect(describeSign(-2,),).toBe('negative',);
            expect(describeSign(5,),).toBe('positive',);
          },
        },),
      ],
    },),
  ],
},);

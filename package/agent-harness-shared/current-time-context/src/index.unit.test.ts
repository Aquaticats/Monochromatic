import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { formatTimeContext, } from '@monochromatic-dev/agent-harness-shared-current-time-context';

await describe({
  name: '',
  children: [
    describe({
      name: formatTimeContext.name,
      children: [
        it({
          name: 'zero-pads single-digit hours and minutes',
          fn: async function testSingleDigitHourAndMinute() {
            expect(
              formatTimeContext(new Date(2_026, 4, 1, 7, 5,),),
            )
              .toBe('<time>07:05</time>',);
          },
        },),
        it({
          name: 'renders midnight as 00:00',
          fn: async function testMidnight() {
            expect(
              formatTimeContext(new Date(2_026, 4, 1, 0, 0,),),
            )
              .toBe('<time>00:00</time>',);
          },
        },),
        it({
          name: 'renders typical evening time verbatim',
          fn: async function testTypicalEveningTime() {
            expect(
              formatTimeContext(new Date(2_026, 4, 1, 20, 48,),),
            )
              .toBe('<time>20:48</time>',);
          },
        },),
        it({
          name: 'renders the last minute of the day as 23:59',
          fn: async function testLastMinuteOfDay() {
            expect(
              formatTimeContext(new Date(2_026, 4, 1, 23, 59,),),
            )
              .toBe('<time>23:59</time>',);
          },
        },),
      ],
    },),
  ],
},);

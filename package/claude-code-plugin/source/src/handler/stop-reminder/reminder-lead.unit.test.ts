import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  dismissalLead,
  questionLead,
  uncertaintyLead,
} from './reminder-lead.ts';

await describe({
  name: 'reminder leads',
  children: [
    describe({
      name: uncertaintyLead.name,
      children: [
        it({
          name: 'names the hedge in parentheses and ends with a full stop',
          fn: async () => {
            expect(uncertaintyLead('probably',),).toBe('Your response contains uncertain language ("probably").',);
          },
        },),
      ],
    },),
    describe({
      name: dismissalLead.name,
      children: [
        it({
          name: 'names the dismissal in parentheses and ends with a full stop',
          fn: async () => {
            expect(dismissalLead("project doesn't use",),)
              .toBe('Your response contains an uncited categorical dismissal ("project doesn\'t use").',);
          },
        },),
      ],
    },),
    describe({
      name: questionLead.name,
      children: [
        it({
          name: 'names the question sentence in parentheses and ends with a full stop',
          fn: async () => {
            expect(questionLead('Want me to commit?',),)
              .toBe('Your response ends with a question to the user ("Want me to commit?").',);
          },
        },),
      ],
    },),
  ],
},);

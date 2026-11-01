import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  formatStartLine,
  formatWaitHint,
} from './report.ts';

await describe({
  name: '',
  children: [
    describe({
      name: formatStartLine.name,
      children: [
        it({
          name: 'reports a started unit',
          fn: async () => {
            expect(formatStartLine({ unit: 'u', status: 0, stderr: '' })).toBe('u: started');
          },
        }),
        it({
          name: 'reports the trimmed error text of a failed start',
          fn: async () => {
            expect(formatStartLine({ unit: 'u', status: 1, stderr: 'boom\n' })).toBe('u: boom');
          },
        }),
        it({
          name: 'reports the status when a failed start printed nothing',
          fn: async () => {
            expect(formatStartLine({ unit: 'u', status: 1, stderr: '' })).toBe('u: systemd-run ended with status 1');
          },
        }),
      ],
    }),
    describe({
      name: formatWaitHint.name,
      children: [
        it({
          name: 'lists every unit in one systemctl command',
          fn: async () => {
            expect(formatWaitHint(['a', 'b'])).toBe('wait with: systemctl --user is-active a b');
          },
        }),
      ],
    }),
  ],
});

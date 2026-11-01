/**
 Forwarded read commands: the wrapper only classifies them and runs real Git. `native-perf.ts` holds each to the
 forwarded-command bound.
 */
export const forwarded = [
  {
    name: 'rev-parse',
    arguments_: [
      'rev-parse',
      'HEAD',
    ],
  },
  {
    name: 'status',
    arguments_: [
      'status',
      '--porcelain',
    ],
  },
  {
    name: 'log',
    arguments_: [
      'log',
      '-1',
      '--format=%H',
    ],
  },
  {
    name: 'diff-stat',
    arguments_: [
      'diff',
      '--stat',
    ],
  },
];

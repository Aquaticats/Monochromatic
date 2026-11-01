import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  campaignCommand,
  campaignContainerfile,
  evidenceJson,
  exitRecord,
  isContentAddressedId,
} from './campaign-files.ts';

/**
 Image ID the tests use: 64 hexadecimal digits.
 */
const imageId = 'a'.repeat(64);

await describe({
  name: '',
  children: [
    describe({
      name: campaignCommand.name,
      children: [
        it({
          name: 'builds the fixed options, then the exclusions, then the product tail, as the pinned scripts did',
          fn: async () => {
            expect(campaignCommand({
              outputDirectory: '/work/mutation-report',
              timeoutSeconds: 180,
              buildTimeoutSeconds: 300,
              additionalExclusions: [],
              tail: ['--file', 'src/a.rs', '--re', 'x'],
            })).toEqual([
              'cargo',
              'mutants',
              '--in-place',
              '--baseline',
              'run',
              '--build-timeout',
              '300',
              '--timeout',
              '180',
              '--no-config',
              '--no-shuffle',
              '--test-tool',
              'nextest',
              '--output',
              '/work/mutation-report',
              '--cargo-arg=--offline',
              '--cargo-arg=--locked',
              '--exclude-re',
              String.raw`replace \+= with \*=`,
              '--exclude-re',
              'replace -= with /=',
              '--file',
              'src/a.rs',
              '--re',
              'x',
            ]);
          },
        }),
        it({
          name: 'leaves out --no-config only when the crate\'s own config applies',
          fn: async () => {
            const base = {
              outputDirectory: '/r',
              timeoutSeconds: 1,
              buildTimeoutSeconds: 2,
              additionalExclusions: [],
              tail: [],
            };
            expect(campaignCommand(base)).toContain('--no-config');
            expect(campaignCommand({ ...base, useProjectConfig: true })).not.toContain('--no-config');
          },
        }),
        it({
          name: 'puts a product exclusion after the repository-wide ones and before the tail',
          fn: async () => {
            const command = campaignCommand({
              outputDirectory: '/r',
              timeoutSeconds: 1,
              buildTimeoutSeconds: 2,
              additionalExclusions: ['extra'],
              tail: ['--tail'],
            });
            expect(command.slice(-3)).toEqual(['--exclude-re', 'extra', '--tail']);
          },
        }),
      ],
    }),
    describe({
      name: campaignContainerfile.name,
      children: [
        it({
          name: 'writes the layering Containerfile byte for byte as the pinned scripts did',
          fn: async () => {
            expect(campaignContainerfile({
              baseImage: imageId,
              command: ['cargo', 'mutants', '--file', 'a b.rs'],
            })).toBe([
              '# The tested image ID binds this campaign to an exact source snapshot.',
              `FROM ${imageId}`,
              'COPY cargo-mutants /usr/local/cargo/bin/cargo-mutants',
              'RUN ["cargo", "mutants", "--version"]',
              'ENV CARGO_NET_OFFLINE=true',
              'CMD ["cargo","mutants","--file","a b.rs"]',
              '',
            ].join('\n'));
          },
        }),
      ],
    }),
    describe({
      name: isContentAddressedId.name,
      children: [
        it({
          name: 'accepts exactly 64 lowercase hexadecimal digits',
          fn: async () => {
            expect(isContentAddressedId(imageId)).toBe(true);
            expect(
              isContentAddressedId('0123456789abcdef'.repeat(4)),
            ).toBe(true);
          },
        }),
        ...[
          ['an empty text', ''],
          ['a short ID', 'a'.repeat(63)],
          ['a long ID', 'a'.repeat(65)],
          ['an uppercase digit', `A${'a'.repeat(63)}`],
          ['a non-hexadecimal letter', `g${'a'.repeat(63)}`],
          ['an ID with a newline', `${'a'.repeat(63)}\n`],
        ].map(function rejection([name, text]) {
          return it({
            name: `rejects ${String(name)}`,
            fn: async () => {
              expect(
                isContentAddressedId(String(text)),
              ).toBe(false);
            },
          });
        }),
      ],
    }),
    describe({
      name: evidenceJson.name,
      children: [
        it({
          name: 'indents by two spaces and ends with one newline',
          fn: async () => {
            expect(evidenceJson({ a: [1], b: 'x' })).toBe('{\n  "a": [\n    1\n  ],\n  "b": "x"\n}\n');
          },
        }),
      ],
    }),
    describe({
      name: exitRecord.name,
      children: [
        it({
          name: 'keeps the status and signal and leaves out an absent copy error',
          fn: async () => {
            expect(
              evidenceJson(exitRecord({
                status: 0,
                signal: undefined,
                reportCopied: true,
              })),
            ).toBe('{\n  "status": 0,\n  "signal": null,\n  "reportCopied": true\n}\n');
          },
        }),
        it({
          name: 'records a signal and a copy error',
          fn: async () => {
            expect(
              evidenceJson(exitRecord({
                status: null,
                signal: 'SIGKILL',
                reportCopied: false,
                reportCopyError: 'no such file',
              })),
            ).toBe('{\n  "status": null,\n  "signal": "SIGKILL",\n  "reportCopied": false,\n  "reportCopyError": "no such file"\n}\n');
          },
        }),
      ],
    }),
  ],
});

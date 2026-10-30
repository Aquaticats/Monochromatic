import type { ArgumentSet } from './golden-argument-sets.ts';

/**
 Environment of the engine campaign scenarios: name the image tag.
 */
export const coreEnvironment: Readonly<Record<string, string>> = { FORBIDDEN_REGEX_IMAGE_TAG: 'golden' };

/**
 Engine campaign scenarios (the mount-free campaign that replaced the host-mounted run). Captured when the engine
 joined the shared flow, so they pin that flow's behavior for the engine rather than a previous script's.
 */
export const coreCampaignSets: readonly ArgumentSet[] = [
  [
    'default',
    [],
    {}
  ],
  [
    'scoped-files-and-timeout',
    [
      '--file',
      'src/dfa.rs',
      '--file',
      'src/parse_*.rs',
      '--timeout',
      '60'
    ],
    {}
  ],
  [
    'examine',
    [
      '--examine',
      'replace foo|bar::baz'
    ],
    {}
  ],
  [
    'container-limits',
    [
      '--memory',
      '4g',
      '--cpus',
      '3',
      '--pids-limit',
      '64'
    ],
    {}
  ],
  [
    'reject-file-outside-src',
    [
      '--file',
      'tests/integration.rs'
    ],
    {}
  ],
  [
    'reject-timeout-zero',
    [
      '--timeout',
      '0'
    ],
    {}
  ],
  [
    'reject-memory',
    [
      '--memory',
      '0'
    ],
    {}
  ],
  [
    'reject-unknown-option',
    [
      '--jobs',
      '4'
    ],
    {}
  ],
];

/**
 Engine snapshot image build scenarios.
 */
export const coreImageSets: readonly ArgumentSet[] = [
  [
    'default',
    [],
    {}
  ],
  [
    'container-limits',
    [
      '--memory',
      '4g',
      '--cpus',
      '3'
    ],
    {}
  ],
  [
    'reject-pids-limit',
    [
      '--pids-limit',
      '64'
    ],
    {}
  ],
  [
    'reject-unknown-option',
    [
      '--jobs',
      '4'
    ],
    {}
  ],
];

import type { ReleaseLabel } from './crates-io-version-label.ts';

/**
 One version and the label the owner's rule gives it.
 */
export type LabelCase = {
  readonly version: string;
  readonly label: ReleaseLabel;
};

/**
 The owner's rule, as versions: a plain number and `-rc.N` go to crates.io; `-alpha.N` and `-beta.N` never do. The list also holds the forms
 the rule does not name (a label without a number, a number after the number, another label, other letter case, a
 leading zero, a text that is no version at all), which must never reach crates.io, because a crates.io publish cannot
 be undone. Build metadata after a plus sign takes no part. The workflow's decision action and the release tool's
 decision are both compared with this list.
 */
export const labelCases: readonly LabelCase[] = [
  { version: '0.2.6', label: 'stable' },
  { version: '1.0.0', label: 'stable' },
  { version: '10.20.30', label: 'stable' },
  { version: '1.0.0+build.5', label: 'stable' },
  { version: '1.0.0-rc.1', label: 'rc' },
  { version: '1.0.0-rc.10', label: 'rc' },
  { version: '1.0.0-rc.0', label: 'rc' },
  { version: '1.0.0-rc.1+build-5.sha', label: 'rc' },
  { version: '0.2.7-alpha.1', label: 'alpha' },
  { version: '0.2.7-alpha.10', label: 'alpha' },
  { version: '0.2.7-alpha.0+x', label: 'alpha' },
  { version: '0.2.7-beta.1', label: 'beta' },
  { version: '0.2.7-beta.12', label: 'beta' },
  { version: '1.0.0-alpha', label: 'unlabelled' },
  { version: '1.0.0-beta', label: 'unlabelled' },
  { version: '1.0.0-rc', label: 'unlabelled' },
  { version: '1.0.0-alpha.1.2', label: 'unlabelled' },
  { version: '1.0.0-rc.1.2', label: 'unlabelled' },
  { version: '1.0.0-alpha.beta', label: 'unlabelled' },
  { version: '1.0.0-rc.x', label: 'unlabelled' },
  { version: '1.0.0-rc.01', label: 'unlabelled' },
  { version: '1.0.0-RC.1', label: 'unlabelled' },
  { version: '1.0.0-Alpha.1', label: 'unlabelled' },
  { version: '1.0.0-rcx.1', label: 'unlabelled' },
  { version: '1.0.0-alpha-1', label: 'unlabelled' },
  { version: '1.0.0-dev.1', label: 'unlabelled' },
  { version: '1.0.0-nightly.3', label: 'unlabelled' },
  { version: '1.0.0-0', label: 'unlabelled' },
  { version: '1.0.0-rc.1-alpha.2', label: 'unlabelled' },
  { version: '1.0.0-alpha.1+', label: 'unlabelled' },
  { version: '1.0.0-rc.1+a..b', label: 'unlabelled' },
  { version: '1.0.0-rc.1+a+b', label: 'unlabelled' },
  { version: '', label: 'unlabelled' },
  { version: '1', label: 'unlabelled' },
  { version: '1.0', label: 'unlabelled' },
  { version: '1.0.0.0', label: 'unlabelled' },
  { version: '01.0.0', label: 'unlabelled' },
  { version: 'v1.0.0', label: 'unlabelled' },
  { version: ' 1.0.0', label: 'unlabelled' },
  { version: '1.0.0 ', label: 'unlabelled' },
  { version: '1.0.0\n', label: 'unlabelled' },
  { version: '1.0.0-rc.1\n', label: 'unlabelled' },
  { version: '1.0.0-rc.1; touch x', label: 'unlabelled' },
  { version: '1.0.0-alpha.1 rc.1', label: 'unlabelled' },
  { version: 'latest', label: 'unlabelled' },
];

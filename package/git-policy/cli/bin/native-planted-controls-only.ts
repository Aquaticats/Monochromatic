#!/usr/bin/env node
/**
 Run only the native wrapper's planted controls over the tested image, and record their pass.

 This is what the extra unit of `native:mutation:parallel` runs. It uses the same limits, scratch and evidence layout
 as a campaign, so its record (`planted-controls-passed.json` in a `native-mutation-planted-*` evidence directory) is
 found by every campaign over the same image. The log ends with the record path, or with the control that was not
 noticed.
 */
import {
  mkdir,
  mkdtemp,
  mkdtempDisposable,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  runLimitArguments,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import { parseContainerLimitOptions } from './container-limits.ts';
import { campaignPrefix } from './native-gate-proof.ts';
import { ensurePlantedControls } from './native-planted-controls-ensure.ts';
import { runPlantedControls } from './native-planted-controls.ts';
import {
  imageOptionOf,
  nativeTestImageRepository,
} from './native-planted-parallel.ts';
import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

/**
 Tag of the tested image; the launcher sets it, as for a campaign, and it names the image only for a hand-run task.
 */
const imageTag = process.env
  .GIT_POLICY_NATIVE_IMAGE_TAG
  ?? 'development';
/**
 Container limits parsed from the command line (`--memory`, `--cpus`, `--pids-limit`).
 */
const {
  limits,
  rest,
} = parseContainerLimitOptions({
  options: process.argv
    .slice(2),
  acceptPidsLimit: true,
});
/**
 Directory that keeps the evidence of the gate and of every campaign.
 */
const evidenceRoot = join(
  process.cwd(),
  'target',
  'verification',
);
/**
 The full image ID the launcher gave as `--image`, as the only element, or none for a hand-run task.
 */
const [givenImage] = imageOptionOf(rest);
/**
 Full ID of the tested image, which the record is keyed by. The launcher's ID is used as given, because the tag may move
 while this unit waits for a slot; only a hand-run task resolves the tag.
 */
const image = givenImage
  ?? (await runCommand({
    command: 'podman',
    args: [
      'image',
      'inspect',
      `${nativeTestImageRepository}:${imageTag}`,
      '--format',
      '{{.Id}}',
    ],
    capture: true,
  })).stdout
    .trim();
await mkdir(
  evidenceRoot,
  { recursive: true },
);
/**
 Evidence directory of this run; the `native-mutation-` prefix makes campaigns look in it.
 */
const evidence = await mkdtemp(join(
  evidenceRoot,
  `${campaignPrefix}planted-`,
));
/**
 Scratch directory for the planted sources, inside the package's gitignored `node_modules/.monochromatic`.
 */
const scratchBase = join(
  import.meta.dirname,
  '..',
  'node_modules',
  '.monochromatic',
  'git-policy',
);
await mkdir(
  scratchBase,
  { recursive: true },
);
/**
 Disposable build context, which holds the files copied into the containers.
 */
await using context = await mkdtempDisposable(join(
  scratchBase,
  'monochromatic-git-policy-cli-planted-',
));
try {
  await ensurePlantedControls({
    setting: {
      base: image,
      limits: [
        '--init',
        '--network=none',
        ...runLimitArguments({
          limits,
          withPidsLimit: true,
        }),
      ],
      context: context.path,
      evidence,
    },
    evidenceRoot,
    evidencePrefix: campaignPrefix,
    skipRequested: false,
    requirePublished: true,
    run: runPlantedControls,
  });
}
catch (error) {
  // The unit's log is the only place a person learns that the controls failed, so it ends with the reason.
  console.error(`Planted controls FAILED over image ${image}: ${error instanceof NativeVerificationError ? error.message : String(error)}`);
  // A failed unit must show as failed to systemd, whatever stopped it.
  process.exitCode = 1;
}

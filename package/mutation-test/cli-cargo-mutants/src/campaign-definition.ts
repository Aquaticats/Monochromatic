import type { ContainerLimits } from './container-limits.ts';

/**
 Facts about a finished set-up that a product's manifest records.
 */
export type ManifestFacts = {
  readonly baseImage: string;
  readonly campaignImage: string;
  readonly toolSha256: string;
  readonly container: string;
  /**
   The command the container runs, which has `--baseline skip` when the product proved the baseline.
   */
  readonly command: readonly string[];
  /**
   The product's proof that the tested image already passed the whole suite, when the baseline was skipped.
   */
  readonly baselineProof?: string;
  /**
   The pass this manifest describes in a two-pass campaign; absent in a one-pass campaign.
   */
  readonly pass?: 1 | 2;
};

/**
 What a product's hook before the image build receives.
 */
export type BeforeBuildContext = {
  readonly baseImage: string;
  /**
   Podman arguments that bound a container: no init process or network, the memory, the CPUs and the process cap.
   */
  readonly containerArguments: readonly string[];
  readonly contextPath: string;
  readonly evidence: string;
  /**
   Mutants that earlier attempts of this campaign scored: zero on the first attempt, more when the campaign resumes,
   which tells the hook that whatever it proves before scoring was proved by an earlier attempt.
   */
  readonly scoredMutants: number;
};

/**
 Everything that makes a product's campaign its own.
 */
export type CampaignDefinition = {
  /**
   Tag of the already tested image the campaign layers over.
   */
  readonly testImage: string;
  /**
   Tag given to the campaign image.
   */
  readonly mutationImage: string;
  /**
   Text appended to the message when the tested image does not resolve, such as the task that builds it.
   */
  readonly testImageHint: string;
  /**
   Directory under which the throwaway build context is created.
   */
  readonly scratchBase: string;
  /**
   Name prefix of the build-context directory.
   */
  readonly contextPrefix: string;
  /**
   Directory that keeps the evidence of every campaign.
   */
  readonly evidenceRoot: string;
  /**
   Name prefix of this campaign's evidence directory.
   */
  readonly evidencePrefix: string;
  readonly limits: ContainerLimits;
  /**
   Report directory inside the container.
   */
  readonly containerReport: string;
  /**
   The campaign command with its baseline run; it names the campaign, so its key does not change when the baseline is
   later skipped.
   */
  readonly command: readonly string[];
  /**
   Pass 1 of a two-pass campaign, which runs only the tests that reach the mutated files; absent for a one-pass
   campaign. Its command is {@link CampaignDefinition.command} with the test selection added (its baseline setting and
   everything else equal), and it is not part of the campaign's key: see `runCampaign`.
   */
  readonly passOne?: {
    readonly command: readonly string[];
  };
  /**
   Look for proof that the tested image already passed the whole suite, so the campaign's own baseline run adds nothing.
   It is called with the full image ID the campaign resolved, and answers a short description of the proof (where it
   was found) as the only element, or no element when there is none. With a proof the container runs `--baseline skip`; with none the baseline
   runs. The answer must never be a guess: a baseline that fails in the container makes every mutant look caught.
   */
  readonly baselineProof?: (context: { readonly baseImage: string }) => Promise<readonly string[]>;
  /**
   Work a product does after the tested image is known and before the campaign image is built.
   */
  readonly beforeBuild?: (context: BeforeBuildContext) => Promise<void>;
  /**
   Record written as `manifest.json`.
   */
  readonly manifest: (facts: ManifestFacts) => unknown;
  /**
   Check a product runs on the evidence directory once the campaign passed and its report was copied.
   */
  readonly afterReport?: (evidence: string) => Promise<void>;
};

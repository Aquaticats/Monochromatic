import { CargoMutantsToolingError } from './error.ts';
import { isContentAddressedId } from './campaign-files.ts';
import { mapInSequence } from './in-sequence.ts';
import type { LauncherOptions } from './options.ts';
import type { UnitPlan } from './plan.ts';
import {
  type CapturedResult,
  runCaptured,
} from './run-captured.ts';
import { buildUnitEnvironment } from './unit-environment.ts';

/**
 Characters of an image ID that name it in a line for a person.
 */
const shortIdLength = 12;

/**
 Runs one `podman` call and answers how it ended; `runCaptured` in production, a stand-in in tests.
 */
export type ImageCaller = (call: {
  readonly args: readonly string[];
  readonly environment: Readonly<Record<string, string>>;
}) => Promise<CapturedResult>;

/**
 What looking up one image reference gave: its full ID, or why there is none.
 */
type ImageAnswer = { readonly image: string } | { readonly failure: string };

/**
 The environment the launcher looks at images with: the assignments a unit with that tag gets (the tag, the person's
 `--setenv` entries, and the launcher's own `TMPDIR`, `PATH` and `HOME` last), and nothing inherited from the launcher's
 shell, so it looks in the container storage the units will use. Only `XDG_RUNTIME_DIR`, which rootless Podman needs and
 the user service manager sets to the same value for a unit, is taken over.

 @param options - the launcher options of this launch

 @param tag - the image tag being looked at

 @param runtimeDirectory - the launcher's `XDG_RUNTIME_DIR`, when it has one

 @returns the variables, later assignments replacing earlier ones of the same name

 @example
 ```ts
 inspectionEnvironment({ options, tag: 'w50', runtimeDirectory: '/run/user/1000' });
 ```
 */
export function inspectionEnvironment({
  options,
  tag,
  runtimeDirectory,
}: {
  readonly options: LauncherOptions;
  readonly tag: string;
  readonly runtimeDirectory?: string;
}): Readonly<Record<string, string>> {
  return Object.fromEntries([
    ...(runtimeDirectory === undefined ? [] : [`XDG_RUNTIME_DIR=${runtimeDirectory}`]),
    ...buildUnitEnvironment({
      options,
      tag,
    }),
  ].map(function pairOf(entry) {
    /**
     Position of the first equals sign, which ends the name.
     */
    const separator = entry.indexOf('=');
    return [
      entry.slice(
        0,
        separator,
      ),
      entry.slice(separator + 1),
    ];
  }));
}

/**
 Reference of the image a tag names.

 @param repository - repository part of the reference

 @param tag - image tag

 @returns `<repository>:<tag>`

 @example
 ```ts
 imageReferenceOf({ repository: 'localhost/product-test', tag: 'w50' });
 ```
 */
export function imageReferenceOf({
  repository,
  tag,
}: {
  readonly repository: string;
  readonly tag: string;
}): string {
  return `${repository}:${tag}`;
}

/**
 Run `podman` with the arguments in the environment, through `runCaptured`.

 @param args - complete Podman arguments

 @param environment - the whole environment of the call

 @returns how the call ended
 */
async function callPodman({
  args,
  environment,
}: {
  readonly args: readonly string[];
  readonly environment: Readonly<Record<string, string>>;
}): Promise<CapturedResult> {
  return await runCaptured({
    program: 'podman',
    args,
    capture: true,
    environment,
  });
}

/**
 Look up the full ID of an image reference without changing anything; a failure is an answer, not an error.

 @param reference - image reference

 @param environment - environment of the call

 @param call - runs `podman`

 @returns the full image ID, or the text of why the reference does not resolve
 */
async function lookUpImage({
  reference,
  environment,
  call,
}: {
  readonly reference: string;
  readonly environment: Readonly<Record<string, string>>;
  readonly call: ImageCaller;
}): Promise<ImageAnswer> {
  /**
   The Podman answer.
   */
  const inspected = await call({
    args: [
      'image',
      'inspect',
      reference,
      '--format',
      '{{.Id}}',
    ],
    environment,
  });
  /**
   The image ID Podman printed.
   */
  const image = inspected.stdout
    .trim();
  return (inspected.status === 0) && isContentAddressedId(image)
    ? { image }
    : {
      failure: inspected.stderr
        .trim()
        || `podman answered status ${String(inspected.status)} and ${JSON.stringify(image)} instead of a full image ID`,
    };
}

/**
 What every helper of one preparation needs: the launch, the runtime directory and the way to call `podman`.
 */
type ImageContext = {
  readonly options: LauncherOptions;
  readonly runtimeDirectory?: string;
  readonly call: ImageCaller;
};

/**
 The image every unit tag is made from.
 */
type SourceImage = {
  readonly reference: string;
  readonly image: string;
};

/**
 A unit tag that did not resolve, and why.
 */
type MissingTag = {
  readonly reference: string;
  readonly failure: string;
};

/**
 Reference of the image a tag names in the launch's repository.

 @param context - the launch

 @param tag - image tag

 @returns `<repository>:<tag>`
 */
function referenceInLaunch({
  context,
  tag,
}: {
  readonly context: ImageContext;
  readonly tag: string;
}): string {
  /**
   Repository of the launch's images.
   */
  const { imageRepository: repository } = context.options;
  return imageReferenceOf({
    repository,
    tag,
  });
}

/**
 Environment of the `podman` calls about one tag.

 @param context - the launch

 @param tag - image tag the unit with it would get in its environment

 @returns the environment, as the units get it
 */
function environmentOfTag({
  context,
  tag,
}: {
  readonly context: ImageContext;
  readonly tag: string;
}): Readonly<Record<string, string>> {
  return inspectionEnvironment({
    options: context.options,
    tag,
    ...(context.runtimeDirectory === undefined ? {} : { runtimeDirectory: context.runtimeDirectory }),
  });
}

/**
 Look up the image a tag of the launch's repository names, without changing anything.

 @param context - the launch

 @param tag - image tag

 @returns the full image ID, or the text of why the tag does not resolve
 */
async function lookUpTag({
  context,
  tag,
}: {
  readonly context: ImageContext;
  readonly tag: string;
}): Promise<ImageAnswer> {
  return await lookUpImage({
    reference: referenceInLaunch({
      context,
      tag,
    }),
    environment: environmentOfTag({
      context,
      tag,
    }),
    call: context.call,
  });
}

/**
 The refusal of a launch whose unit tags do not exist, naming each and the commands that make them.

 @param context - the launch

 @param missing - the tags that did not resolve

 @returns the error to throw
 */
function missingTagsError({
  context,
  missing,
}: {
  readonly context: ImageContext;
  readonly missing: readonly MissingTag[];
}): CargoMutantsToolingError {
  /**
   Repository of the launch's images.
   */
  const { imageRepository: repository } = context.options;
  return new CargoMutantsToolingError([
    `Nothing was started: ${missing.length === 1 ? 'one unit image tag does' : 'some unit image tags do'} not exist, and a unit whose image is missing dies at once with "image not known".`,
    ...missing.map(function missingLine({
      reference,
      failure,
    }: MissingTag) {
      return `- ${reference} (podman: ${failure})`;
    }),
    'Make each with `podman tag` of the image the gate built:',
    ...missing.map(function fixLine({ reference }: MissingTag) {
      return `  podman tag ${referenceInLaunch({
        context,
        tag: '<gate-tag>',
      })} ${reference}`;
    }),
    `Or pass --from-tag <gate-tag> (the tag of the gate image in ${repository}), and the launcher makes every unit's tag itself.`,
  ].join('\n'));
}

/**
 Point one unit's tag at the source image and check that it names it, then say whether that moved the tag.

 @param context - the launch

 @param plan - the unit

 @param source - the source reference and the full ID it names

 @returns a line when the tag held another image before, none otherwise

 @throws CargoMutantsToolingError when `podman tag` fails or the tag does not name the source's ID afterwards
 */
async function tagUnit({
  context,
  plan,
  source,
}: {
  readonly context: ImageContext;
  readonly plan: UnitPlan;
  readonly source: SourceImage;
}): Promise<readonly string[]> {
  /**
   Reference of this unit's image.
   */
  const reference = referenceInLaunch({
    context,
    tag: plan.tag,
  });
  /**
   Command that makes the tag, as a person would type it.
   */
  const command = `podman tag ${source.image} ${reference}`;
  /**
   What the tag named before this launch; the source itself when the unit's tag is the source's.
   */
  const before = reference === source.reference
    ? { image: source.image }
    : await lookUpTag({
      context,
      tag: plan.tag,
    });
  if (reference !== source.reference) {
    /**
     The `podman tag` call.
     */
    const tagged = await context.call({
      args: [
        'tag',
        source.image,
        reference,
      ],
      environment: environmentOfTag({
        context,
        tag: plan.tag,
      }),
    });
    if (tagged.status !== 0)
      throw new CargoMutantsToolingError(`Nothing was started: \`${command}\` failed (status ${String(tagged.status)}): ${tagged.stderr
        .trim()}`);
  }
  /**
   What the tag names now.
   */
  const after = await lookUpTag({
    context,
    tag: plan.tag,
  });
  if (('failure' in after) || (after.image !== source.image))
    throw new CargoMutantsToolingError(`Nothing was started: the tag ${reference} does not name the image ${source.image} that ${source.reference} names (it names ${'failure' in after ? `nothing: ${after.failure}` : after.image}). Make it with \`${command}\`, then launch again.`);
  return ('image' in before) && (before.image !== source.image)
    ? [`Image tag ${reference} moved from image ${before.image} to ${source.image}; the old image stays in storage.`]
    : [];
}

/**
 Make every unit's image tag exist before any unit starts, or refuse the whole launch in plain words.

 A unit that starts without its tag dies at once ("image not known"), while the launcher that started it would still have
 exited 0. So this runs first. Podman is used for two things only: reading an image (`podman image inspect`) and, with
 `fromTag`, `podman tag`. It never removes an image or a tag.

 With `options.fromTag`, the image named by `<repository>:<fromTag>` is the source. Each unit's tag is pointed at it by
 its full ID (so a source tag that moves meanwhile changes nothing), and then looked up again: a tag that does not
 resolve to the source's ID refuses the launch. `podman tag` moves a tag that already exists; a unit tag that held another
 image is named in the returned lines, with both IDs, so a move is never silent. The old image stays in storage.

 Without `fromTag`, every unit's tag must already resolve to a full image ID, and the launch is refused naming each one
 that does not, with the `podman tag` command that makes it.

 @param options - validated launcher options

 @param plans - the units, whose tags are made or checked, in order

 @param runtimeDirectory - the launcher's `XDG_RUNTIME_DIR`, which rootless Podman needs, when it has one

 @param call - runs `podman`; `runCaptured` in production, a stand-in in tests

 @returns lines for the person: one summary, then one per tag that was moved from another image

 @throws CargoMutantsToolingError when the source image is missing, a `podman tag` fails, a tag does not resolve to the
 source's ID afterwards, or (without `fromTag`) a unit tag does not resolve

 @example
 ```ts
 await ensureUnitImages({ options, plans });
 ```
 */
export async function ensureUnitImages({
  options,
  plans,
  runtimeDirectory,
  call = callPodman,
}: {
  readonly options: LauncherOptions;
  readonly plans: readonly UnitPlan[];
  readonly runtimeDirectory?: string;
  readonly call?: ImageCaller;
}): Promise<readonly string[]> {
  /**
   The launch, for the helpers.
   */
  const context: ImageContext = {
    options,
    ...(runtimeDirectory === undefined ? {} : { runtimeDirectory }),
    call,
  };
  if (options.fromTag === undefined) {
    /**
     The units whose tag does not resolve, found one lookup after another.
     */
    const missing = (await mapInSequence({
      items: plans,
      run: async function lookUpUnit(plan): Promise<readonly MissingTag[]> {
        /**
         What this unit's tag resolves to.
         */
        const answer = await lookUpTag({
          context,
          tag: plan.tag,
        });
        return 'failure' in answer ? [{
          reference: referenceInLaunch({
            context,
            tag: plan.tag,
          }),
          failure: answer.failure,
        }] : [];
      },
    }))
      .flat();
    if (missing.length > 0)
      throw missingTagsError({
        context,
        missing,
      });
    return [`Image tags: all ${plans.length} unit tags in ${options.imageRepository} exist.`];
  }
  /**
   Reference of the source image.
   */
  const sourceReference = referenceInLaunch({
    context,
    tag: options.fromTag,
  });
  /**
   What the source reference resolves to.
   */
  const sourceAnswer = await lookUpTag({
    context,
    tag: options.fromTag,
  });
  if ('failure' in sourceAnswer)
    throw new CargoMutantsToolingError(`Nothing was started: the image ${sourceReference} to make the unit tags from does not exist (podman: ${sourceAnswer.failure}). Build the gate image first, or name the tag of one that exists with --from-tag; \`podman images ${options.imageRepository}\` lists them.`);
  /**
   The source image's reference and full ID.
   */
  const source = {
    reference: sourceReference,
    image: sourceAnswer.image,
  };
  /**
   The lines of the tags that were moved from another image, one entry per unit.
   */
  const moves = await mapInSequence({
    items: plans,
    run: async function tagOneUnit(plan) {
      return await tagUnit({
        context,
        plan,
        source,
      });
    },
  });
  return [
    `Image tags: ${plans.length} unit tags in ${options.imageRepository} now name image ${source.image
      .slice(
        0,
        shortIdLength,
      )}, the image of ${sourceReference}.`,
    ...moves.flat(),
  ];
}

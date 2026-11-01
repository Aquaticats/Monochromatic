import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { SYNTHETIC_CHAT_BASE_URL, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { ModelTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  CATALOG_MODEL_IDS,
  compareCatalog,
  decodeModelList,
  formatCatalogReport,
} from './model-catalog-compare.ts';
import {
  LISTING_TIMEOUT_MS,
  readProviderListing,
} from '@monochromatic-dev/translation-repair-insertion/ts';

//region Model catalog print
// Asks the provider what it currently serves and prints how that differs from
// the catalog this pipeline compiles against.
//
// This exists because the drift has already happened and cost silently. Two ids
// were removed on 2026-08-05 after both began answering HTTP 404 "is no longer
// supported", and 404 is not in the transient retry set, so every stage holding
// one of them lost a voice per call and nothing said why. The catalog comment
// records facts "verified live on 2026-07-16", and a hand-verified note is
// exactly the kind of claim that rots without announcing it.
//
// It also answers a roster question that cannot be answered from the catalog
// alone: whether any model exists that holds NO role in a run. Critics, panel
// and judges are all the same six, so no judge is independent of the issue it
// would judge, and a genuinely independent judge needs a model the run does not
// already use.
//
// Read-only. Prints ids and nothing else: the key is never rendered.

/**
 Where the provider lists what it serves.
 */
const MODELS_URL = `${SYNTHETIC_CHAT_BASE_URL}/models`;

/**
 Reads the models a listing names, refusing a listing that is not shaped as a
 model list.

 @param body - parsed listing

 @returns Every model listed

 @throws {@link StatedRefusalError} When the listing carries no `data` array
 or an entry without a string id, with the shape the report reads named and
 nothing of the listing quoted

 @example
 ```ts
 const served = servedModelsOf({ body, },);
 ```
 */
function servedModelsOf({ body, }: { readonly body: unknown; },): ReturnType<typeof decodeModelList> {
  try {
    return decodeModelList({ body, },);
  } catch (error) {
    if (!(error instanceof ArtifactParseError)) {
      throw new Error(
        'unreachable: decodeModelList threw a value that is not an ArtifactParseError, '
          + 'though it throws no other class',
        { cause: error, },
      );
    }

    // The shape failure stays the cause; its words are not repeated, so the
    // message is ours alone and names the shape the report reads.
    throw new StatedRefusalError({
      says: `${MODELS_URL} answered a listing this report cannot read: it reads an object whose data `
        + 'is an array of entries that each carry a string id',
      cause: error,
    },);
  }
}

/**
 Reports how the provider's current offering differs from the catalog.

 @param env - environment the key is read from: `process.env` in a run

 @param transport - transport the listing is fetched over: `fetchTransport` in a run

 @throws {@link StatedRefusalError} when the key is absent, so the failure names
 the fix rather than surfacing as an authentication error, and when the
 provider answers with a status that is not a success

 @example
 ```ts
 await printModelCatalog({ env: process.env, transport: fetchTransport, },);
 ```
 */
export async function printModelCatalog(
  {
    env,
    transport,
  }: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly transport: ModelTransport;
  },
): Promise<void> {
  /**
   Synthetic API key, resolved by name from the mise-injected env.
   */
  const apiKey = env
    .TRANSLATION_REPAIR_SYNTHETIC_API_KEY
    ?? '';
  if (apiKey === '')
    throw new StatedRefusalError({
      says: 'TRANSLATION_REPAIR_SYNTHETIC_API_KEY is not set; run under mise so sops injects it',
    },);

  /**
   The listing as the provider answered it.
   */
  const body = await readProviderListing({
    url: MODELS_URL,
    apiKey,
    transport,
    timeoutMs: LISTING_TIMEOUT_MS,
    statusRefusal: function says({ status, },): string {
      return `${MODELS_URL} answered ${String(status,)}. The reason phrase is dropped `
        + 'rather than repeated: it is the provider\'s wording, and this message promises to '
        + 'carry only ours.';
    },
  },);

  /**
   Every model the provider currently serves, aliases included.
   */
  const served = servedModelsOf({ body, },);

  /**
   Drift in both directions, plus the aliases held out of the catalog.
   */
  const comparison = compareCatalog({
    served,
    catalog: CATALOG_MODEL_IDS,
  },);

  // Raw output, deliberately: this is a report a human reads, not pipeline
  // logging, and the tagged logger would wrap every line in a prefix. That is
  // the stated exception to the tagged-logger rule, and it needs no
  // suppression: the corpus-run scripts are all shaped this way.
  console.log(formatCatalogReport({ comparison, },),);
}

//endregion Model catalog print

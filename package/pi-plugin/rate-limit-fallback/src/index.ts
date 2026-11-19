/**
 Pi extension entry for rate-limit fallback between the declared writer and reviewer members.
 @module
 */

import {
  lazyStream,
  type Api,
  type AssistantMessageEventStream,
  type Model,
  type Provider,
  type SimpleStreamOptions,
  type TranscriptContext,
} from '@earendil-works/pi-ai';
import type { ExtensionAPI, } from '@earendil-works/pi-coding-agent';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { dispatchRequest, PoolDispatchError, type RoutingSession, } from './dispatch-request.ts';
import { MODELS, PROVIDER_ID, } from './model-pools.ts';

export { PROVIDER_ID, } from './model-pools.ts';

//region Register the provider before startup model selection

/** Logger for session binding and registration, never credential or transcript data. */
const l = tagged({ tag: 'rate-limit-fallback', },);

/**
 Register the keyless pool provider and bind it to the current Pi session.
 Both stream entry points use provider-neutral options because the picker cannot know a member's protocol.
 @param pi - host extension API
 */
export default function rateLimitFallbackExtension(pi: ExtensionAPI,): void {
  const rl = tagged({ tag: rateLimitFallbackExtension.name, l, },);
  let session: RoutingSession | undefined;

  /**
   Bridge Pi's synchronous stream entry to asynchronous member dispatch.
   @param model - selected pool model
   @param context - normalized transcript
   @param options - request options
   @returns stream of the eventual member, with no events from discarded attempts
   */
  function route(
    model: Model<Api>,
    context: TranscriptContext,
    options?: SimpleStreamOptions,
  ): AssistantMessageEventStream {
    return lazyStream(model, async function prepare() {
      if (session === undefined)
        throw new PoolDispatchError('The rate-limit-fallback provider requires a started Pi session.',);
      return dispatchRequest({ session, model, context, options, },);
    }, undefined, options?.signal,);
  }

  const provider: Provider = {
    id: PROVIDER_ID,
    name: 'Rate limit fallback',
    auth: {
      apiKey: {
        name: 'Keyless pool',
        async resolve() {
          return { auth: {}, source: PROVIDER_ID, };
        },
      },
    },
    getModels() {
      return MODELS;
    },
    stream: route,
    streamSimple: route,
  };
  pi.registerProvider(provider,);
  rl.debug(`Registered ${PROVIDER_ID}`,);
  pi.on('session_start', function bindRegistry(_event, context,) {
    session = { registry: context.modelRegistry, members: new Map(), };
    rl.debug('Bound session model registry',);
  },);
}

//endregion Register the provider before startup model selection

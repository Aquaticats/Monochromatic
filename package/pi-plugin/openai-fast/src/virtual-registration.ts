/**
 Genuine virtual selections derived from every current physical OpenAI model. @module
 */

import {
  getSupportedThinkingLevels,
  type Api,
  type Model,
} from '@earendil-works/pi-ai';
import type {
  ExtensionAPI,
  ModelRouteRequest,
} from '@earendil-works/pi-coding-agent';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type {
  ForeignBorrowed,
  ForeignHostCapability,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  FAST_PROVIDER,
  PRIORITY_TARGET_PREFIX,
} from './constants.ts';
import { FastModelError, } from './fast-model-error.ts';

//region Registration state

/**
 Module logger records catalog changes without prompts or authentication.
 */
const moduleLogger = tagged({ tag: 'pi-plugin-openai-fast.virtual-registration', },);

/**
 Create a guarded synchronizer whose registration state belongs to this extension factory.
 
 @param pi - host registration capability

 @param fastProvider - companion namespace bound to its original provider
 
 @returns synchronizer without a compatibility list or request-global fast flag
 
 @mutates pi - returned callback registers and removes native virtual selections

 @example
 ```ts
 const synchronize = createFastModelRegistration({ pi });
 synchronize(originalModels);
 ```
 */
export function createFastModelRegistration({
  pi,
  fastProvider = FAST_PROVIDER,
}: {
  readonly pi: ForeignHostCapability<ExtensionAPI>;
  readonly fastProvider?: string;
},): (models: readonly Model<Api>[]) => void {
  /**
   Factory logger owns the catalog synchronization boundary.
   */
  const l = tagged({
    tag: createFastModelRegistration.name,
    l: moduleLogger,
  },);
  /**
   Last successfully registered structural definitions, excluding request data.
   */
  const signatures = new Map<string, string>();
  /**
   Owned reentrancy latch, unrelated to per-request priority selection.
   */
  const state = { synchronizing: false, };

  /**
   Register structural changes while nested catalog reads remain side-effect-free.

   @param input - current original native catalog
   */
  function synchronize(input: ForeignBorrowed<readonly Model<Api>[]>,): void {
    /**
     Synchronization logger extends the factory tag.
     */
    const sl = tagged({
      tag: synchronize.name,
      l,
    },);
    if (state.synchronizing) {
      sl.trace('skipping nested catalog registration',);
      return;
    }
    state.synchronizing = true;
    /**
     Reset the owned latch on every scope exit, including registration errors.
     */
    using guard = {
      [Symbol.dispose]: function releaseRegistration(): void {
        state.synchronizing = false;
      },
    };
      /**
       Only physical original models can be fast routing bases.
       */
    const models = input.filter(function physicalModel(model,) {
      return model.api !== 'pi-virtual';
    },);
      /**
       Native IDs validate namespace collisions before registration changes occur.
       */
    const ids = new Set(models.map(function modelId(model: ForeignBorrowed<Model<Api>>, ) {
      return model.id;
    },),);
    for (const model of models) {
      if (model.id
        .startsWith(PRIORITY_TARGET_PREFIX,)
        || ids.has(`${PRIORITY_TARGET_PREFIX}${model.id}`,)) {
        throw new FastModelError(`OpenAI model "${model.id}" conflicts with the fast extension's internal routing namespace. Rename the configured model or remove the fast extension.`,);
      }
    }
    for (const previous of signatures.keys()) {
      if (!ids.has(previous,)) {
        signatures.delete(previous,);
        pi.unregisterVirtualModel(
          fastProvider,
          previous,
        );
        sl.debug(`removed companion for ${previous}`,);
      }
    }
    for (const model of models) {
        /**
         Reserved physical identity differs from both selected and upstream model IDs.
         */
      const targetId = `${PRIORITY_TARGET_PREFIX}${model.id}`;
        /**
         Virtual capabilities follow the original catalog rather than a compatibility list.
         */
      const definition = {
        provider: fastProvider,
        id: model.id,
        name: `${model.name} Fast`,
        thinkingLevels: getSupportedThinkingLevels(model,),
        input: model.input,
        contextWindow: model.contextWindow,
        maxTokens: model.maxTokens,
      };
        /**
         Only selector metadata participates in structural deduplication.
         */
      const signature = JSON.stringify(definition,);
        /**
         Previous definition supports rollback if synchronous registration rejects.
         */
      const previous = signatures.get(model.id,);
      if (previous === signature)
        continue;
      signatures.set(
        model.id,
        signature,
      );
      try {
        pi.registerVirtualModel({
          ...definition,
          route: function route(
            request: ForeignBorrowed<ModelRouteRequest>,
            ctx,
          ) {
              /**
               Request-routing logger records only reason and model identity.
               */
            const rl = tagged({
              tag: route.name,
              l: sl,
            },);
              /**
               Physical target is always looked up from the current catalog.
               */
            const target = ctx.modelRegistry
              .getModelOfType(
              'chat',
              fastProvider,
              targetId,
              );
            if (target === undefined)
              throw new FastModelError(`${fastProvider} model "${model.id}" has no current priority target. Refresh models or select an available ordinary model.`,);
            rl.debug(`routing ${request.reason} request for ${model.id}`,);
            return {
              model: target,
              thinkingLevel: request.thinkingLevel,
            };
          },
        },);
      }
      catch (error) {
        if (previous === undefined)
          signatures.delete(model.id,);
        else
          signatures.set(
            model.id,
            previous,
          );
        sl.error(`virtual registration failed for ${model.id}: ${String(error,)}`,);
        throw error;
      }
      sl.debug(`registered virtual companion for ${model.id}`,);
    }
    sl.trace(`catalog contains ${ids.size} companion identities`,);
  }
  return synchronize;
}

//endregion

/**
 Compose caller payload customization before enforcing priority. @module
 */
import type {
  Api,
  Model,
  StreamOptions,
} from '@earendil-works/pi-ai';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { PriorityRequestError, } from './priority-error.ts';

//region Payload boundary: foreign callbacks retain native replacement semantics.

/**
 Module logger never receives request content or authentication data.
 */
const l = tagged({ tag: 'openai-fast/priority-payload', },);

/**
 Await caller customization, select its replacement when supplied, and return a priority record.
 Original and replacement records are not written by this function.
 Caller callback mutations remain caller-owned.

 @param payload - native request passed unchanged to caller customization

 @param model - live model passed unchanged to caller customization

 @param onPayload - caller customization whose undefined result keeps the native request

 @returns shallow record copy with priority applied after caller customization

 @mutates payload - caller callback may inspect, mutate, or retain request data and its reachable values

 @mutates model - caller callback may inspect, mutate, or retain live model data

 @mutates onPayload - invoking caller callback can change its captured state

 @throws {@link PriorityRequestError} when selected payload is not a plain or null-prototype record or overrides root JSON serialization

 @throws when caller customization rejects; its original rejection is preserved

 @example
 ```ts
 const body = await forcePriorityPayload({ payload, model, onPayload: options.onPayload });
 ```
 */
export async function forcePriorityPayload({
  payload,
  model,
  onPayload,
}: ForeignHostCapability<{
  readonly payload: unknown;
  readonly model: Model<Api>;
  readonly onPayload?: StreamOptions['onPayload'];
}>,): Promise<Readonly<Record<string, unknown>> & { readonly service_tier: 'priority'; }> {
  /**
   Function logger records callback lifecycle without logging request values.
   */
  const innerL = tagged({
    tag: forcePriorityPayload.name,
    l,
  },);
  innerL.debug(onPayload === undefined ? 'preparing native payload' : 'awaiting caller payload customization',);
  /**
   Undefined preserves the original payload; null is an invalid replacement.
   */
  const replacement = await onPayload?.(
    payload,
    model,
  );
  /**
   Selected request must be a record before priority can be applied.
   */
  const selected = replacement === undefined ? payload : replacement;
  if (((typeof selected) !== 'object') || (selected === null)
    || Array.isArray(selected,)) {
    innerL.error('selected priority request payload is not an object record',);
    throw new PriorityRequestError({
      message: 'Priority request payload must be a non-null object, not an array. Return an object from onPayload, or return undefined to keep the native request.',
    },);
  }
  /**
   Built-in containers and class instances cannot preserve native record replacement semantics.
   */
  const prototype: unknown = Object.getPrototypeOf(selected,);
  if ((prototype !== Object.prototype) && (prototype !== null)) {
    innerL.error('selected priority request payload is not a plain or null-prototype record',);
    throw new PriorityRequestError({
      message: 'Priority request payload must be a plain or null-prototype object. Return an object from onPayload, not a class instance or container, or return undefined to keep the native request.',
    },);
  }
  /**
   Copy before serializer validation so each caller getter runs at most once during preparation.
   */
  const result: Readonly<Record<string, unknown>> & { readonly service_tier: 'priority'; } = {
    ...selected,
    service_tier: 'priority',
  };
  /**
   Root serialization customization is inspected without invoking it.
   */
  const { toJSON, } = result;
  if ((typeof toJSON) === 'function') {
    innerL.error('priority request payload overrides root JSON serialization',);
    throw new PriorityRequestError({
      message: 'Priority request payload must not define a callable toJSON property. Return a plain request object from onPayload so service_tier remains priority when sent.',
    },);
  }
  innerL.debug(replacement === undefined ? 'applying priority to native payload' : 'applying priority to caller replacement',);
  return result;
}

//endregion Payload boundary

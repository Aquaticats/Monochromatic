/**
 Preserve policy advisories while preventing Pi from misreading them as transient errors.
 @module
 */

/**
 Transient spans from Pi AI 1.5.2's `dist/utils/retry.js`, `RETRYABLE_PROVIDER_ERROR_PATTERN`.
 Pi exposes the boolean classifier, not its match spans. Keep this list in step with that classifier
 when updating Pi; the non-retryable quota and billing patterns are not grounds for restarting a turn.
 */
const RETRY_TRIGGERS = new RegExp([
  'overloaded',
  'server_busy',
  'servers are currently busy',
  'currently experiencing high demand',
  'model is at capacity',
  'rate.?limit',
  'too many requests',
  '(?<![a-z0-9])(?:429|500|502|503|504|520|524)(?![a-z0-9])',
  'service.?(?:is.?)?(?:temporarily.?)?unavailable',
  'server.?error',
  'internal.?error',
  'provider.?returned.?(?:an.?)?error',
  'exceeded request buffer limit while retrying upstream',
  'unable to forward upstream',
  'upstream.?request.?failed',
  'upstream_stream_read_error',
  'stream was interrupted',
  'http/?2 stream failed',
  'network.?error',
  'connection.?error',
  'connection.?refused',
  'connection.?lost',
  'other side closed',
  'fetch failed',
  'request to \\S+ failed, reason:',
  'getaddrinfo',
  'ENOTFOUND',
  'EAI_AGAIN',
  'upstream.?connect',
  'reset before headers',
  'socket hang up',
  'socket connection was closed',
  'timed? out',
  'timeout',
  'terminated',
  'websocket.?closed',
  'websocket.?error',
  'ended without',
  'stream ended before message_stop',
  'stream ended before a terminal response event',
  'http2 request did not get a response',
  'stream disconnected before completion',
  'cut short after the reply had begun',
  'pending stream has been canceled',
  'retry delay',
  'you can retry your request',
  'try your request again',
  'please retry your request',
  'ResourceExhausted',
  'subscription_sharing_usage_unavailable',
  'subscription_sharing_user_unavailable',
].join('|',), 'gi',);

/**
 Attribute the policy stop and mask only the spans that Pi treats as retry signals.
 The marker keeps separated text from joining into a new retry trigger.

 @param memberLabel - physical provider and model that stopped
 @param message - original provider policy message
 @returns member-attributed policy advisory safe to show to Pi's retry classifier
 */
export function policyStopMessage({
  memberLabel,
  message,
}: {
  readonly memberLabel: string;
  readonly message: string;
},): string {
  return `${memberLabel} stopped the request under its content policy: ${message.replaceAll(RETRY_TRIGGERS, '[omitted]',)}`;
}

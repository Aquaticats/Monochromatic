/**
 Error classes the router maps onto HTTP statuses.

 Thrown rather than returned so a refusal cannot be silently dropped by a
 caller that forgets to inspect a result field. The Caddy original this Worker
 replaces had no refusal path at all: its template functions coerced
 unparseable input to zero and answered 200, so a caller could receive a value
 drawn from a range it never asked for.

 @module
 */

import { NamedError, } from '@monochromatic-dev/named-error-core/ts';

/* oxlint-disable unicorn/custom-error-definition -- Each class takes its name from `NamedError`, which derives it from the concrete class, so no literal is left to check; `doc/troubleshooting/oxlint-custom-error-name.md` records why the rule cannot be satisfied here. */

//region Errors

/**
 Input the request supplied that this Worker refuses to guess at.

 Maps to 400. Covers a missing or non-canonical `min` or `max`, a repeated
 query parameter, a `min` above its `max`, a range too wide to draw from
 uniformly, and a length segment that looks numeric but falls outside the
 served range. Its message names the offending input and the domain that would
 have been accepted.
 */
export class BadRequestError extends NamedError {}

/**
 Path that matches no route and does not look like an attempt at one.

 Maps to 404. Deliberately distinct from {@link BadRequestError}: a path such
 as `/nope` is unknown rather than malformed, while `/65` shows the caller
 understood the API and asked for too much, so the two deserve different
 statuses and different remediation text. Its message names the unmatched path
 and lists the served routes.
 */
export class NotFoundError extends NamedError {}

//endregion Errors

/* oxlint-enable unicorn/custom-error-definition */

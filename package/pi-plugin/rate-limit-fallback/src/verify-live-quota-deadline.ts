/**
 Read every pool member's live quota endpoint under the shipped deadline, and show that the deadline can fire.
 @module

 The package README states one precondition before rounds rely on the writer pool: a live check of the quota
 read's deadline. This verifier runs it. `doc/quota-read-latency.md` measured the deadline from a live endpoint
 measurement on 2026-11-19; the arm below reads the same endpoints again and reports what the shipped bound did:

 - `live`: one read per pool member per round, bounded by `QUOTA_READ_TIMEOUT_MS`, timed around the whole
   `readQuotaWithinDeadline` call. A member whose provider publishes no quota endpoint resolves with empty
   evidence and sends no request (`README.md`, "Provider sources").
 - `stalled`: the same call with a transport that never answers, which proves the bound can abandon a read,
   so a live arm that finished says something about the endpoint rather than about a probe that cannot hang.

Credentials are resolved only through `ModelRegistry.getApiKeyAndHeaders`, the call the read itself makes. This
file reads no credential file and prints no header, key or token: every line names a member, a round, the
milliseconds it took and how the read ended.

 Run it with `mise run //package/pi-plugin/rate-limit-fallback:verify:quota-deadline`.
 */

import {
  ModelRegistry,
  ModelRuntime,
} from '@earendil-works/pi-coding-agent';
import type {
  FetchFunction,
  Model,
} from '@earendil-works/pi-ai';
import { WRITER_TARGETS, REVIEWER_TARGETS, } from './member-order-fixture.ts';
import issueOnStandInCatalog from './stand-in-pi-fixture.ts';

/**
 Members one pool asks, as the fixture spells the owner's order out.
 */
const POOL_MEMBERS: readonly Model<any>[] = [
  ...WRITER_TARGETS,
  ...REVIEWER_TARGETS,
];

export default issueOnStandInCatalog;

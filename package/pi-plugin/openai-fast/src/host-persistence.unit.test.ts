/**
 Actual JSONL resume and branch restoration retain virtual fast selection.
 
 @module
 */
import { SessionManager, } from '@earendil-works/pi-coding-agent';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { FAST_PROVIDER, registerOpenAIFast, } from '../dist/final/node/index.mjs';
import { fixtureHome, fixtureHost, requireCompanion, } from './host-fixture-session.ts';

//region Persistence: virtual selection outlives native replies and restoration.

await describe({ name: registerOpenAIFast.name, children: [
  it({ name: 'persists fast selection across turns, actual JSONL resume, and branched session restoration', fn: async function verifySessionRestoration() {
    /** Actual session persistence remains confined to independently disposable paths. */
    await using home = await fixtureHome();
    /** Real SDK session manager persists selection changes and canonical native replies. */
    using host = await fixtureHost({ home, },);
    /** Genuine virtual selection is stored rather than its hidden physical target. */
    const companion = requireCompanion({ runtime: host.runtime, id: host.base.id, },);
    await host.session.setModel(companion,);
    host.session.setThinkingLevel('xhigh',);
    await host.session.prompt('First fast turn.',);
    /** Actual first-turn leaf supplies the branch point for SDK restoration. */
    const branchPoint = host.manager.getLeafId();
    await host.session.prompt('Second fast turn.',);
    expect(host.session.model?.api,).toBe('pi-virtual',);
    /** Real JSONL file is reopened through the installed SDK, not a simulated history. */
    const file = host.manager.getSessionFile();
    if ((file === undefined) || (branchPoint === null))
      throw new Error('Actual host did not persist the disposable session.',);
    expect(host.manager.getBranch().filter(function modelChange(entry) {
      return entry.type === 'model_change';
    },).at(-1),).toMatchObject({ provider: FAST_PROVIDER, modelId: host.base.id, },);
    host.session.dispose();
    /** Native restoration must resolve the persisted virtual entry without fallback. */
    const resumed = await host.resume(SessionManager.open(file, home.sessionDir,),);
    expect(resumed.modelFallbackMessage,).toBeUndefined();
    expect(resumed.session.model,).toMatchObject({ api: 'pi-virtual', provider: FAST_PROVIDER, id: host.base.id, },);
    expect(resumed.session.thinkingLevel,).toBe('xhigh',);
    await resumed.session.prompt('Resumed fast turn.',);
    expect(resumed.session.messages.at(-1),).toMatchObject({ stopReason: 'stop', },);
    resumed.session.dispose();
    /** Actual manager creates a new branch from the persisted first-turn leaf. */
    const branching = SessionManager.open(file, home.sessionDir,);
    /** Branch JSONL is restored using exactly the same native host path. */
    const branchedFile = branching.createBranchedSession(branchPoint,);
    if (branchedFile === undefined)
      throw new Error('Actual host did not create the disposable branch.',);
    /** Branched session must retain the fast provider and priority reasoning. */
    const branched = await host.resume(SessionManager.open(branchedFile, home.sessionDir,),);
    expect(branched.modelFallbackMessage,).toBeUndefined();
    expect(branched.session.model?.provider,).toBe(FAST_PROVIDER,);
    await branched.session.prompt('Branched fast turn.',);
    expect(branched.session.messages.at(-1),).toMatchObject({ stopReason: 'stop', },);
    expect(host.source.state.calls,).toHaveLength(4,);
    for (const call of host.source.state.calls) {
      expect(call.model.id,).toBe(host.base.id,);
      expect(call.options,).toMatchObject({ serviceTier: 'priority', reasoningEffort: 'xhigh', },);
    }
  }, },),
], },);

//endregion Persistence

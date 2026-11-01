/**
 Oxlint plugin whose only rule throws while oxlint sets it up, so the fixture runner can be shown the text oxlint
 prints instead of JSON when a plugin fails to load.

 Oxlint sets up every rule of a plugin when it loads the plugin, so this rule lives in a plugin of its own
 rather than beside the rules of the other probe plugin, which would fail every run that loads it.

 @module
 */

/**
 Message of the error the rule raises while oxlint sets it up, searched for by tests.
 */
export const PROBE_CREATE_ONCE_FAILURE_TEXT = 'harness probe plugin failed during createOnce';

/**
 Plugin whose only rule throws while oxlint sets it up.
 */
const plugin = {
  meta: { name: 'harness-probe-create-once', },
  rules: {
    'create-once-throws': {
      createOnce(): never {
        throw new Error(PROBE_CREATE_ONCE_FAILURE_TEXT,);
      },
    },
  },
};

export default plugin;

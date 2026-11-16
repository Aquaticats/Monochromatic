/**
 The protocol tag this program writes on the policy rules and routes it creates.

 A `suppress_prefixlength 0` rule names no interface and no table,
 so without a tag a teardown cannot tell the rule this program created from one
 another tunnel or an administrator installed.
 The tag is the one the bypass routes and rules already carry,
 so the rules this program created are the ones carrying it.

 @module
 */

import { BYPASS_ROUTE_PROTOCOL, } from './tunnel-bypass-types.ts';

/**
 The tag, in one place, so the modules that write and remove a rule agree on it.
 */
export const POLICY_RULE_PROTOCOL: number = BYPASS_ROUTE_PROTOCOL;

/**
 * Runtime-authoritative loaded configuration validation.
 *
 * @module
 */
import * as v from 'valibot';
import type {
  PolicySeverity,
  PolicyTrigger,
} from '../api/policy-types.ts';
import { BUILT_IN_POLICIES, } from '../policy-engine/built-ins.ts';
import type { RuntimePolicyDefinition, } from '../policy-engine/types.ts';

/**
 * Loaded configuration failed runtime validation.
 */
export class ConfigValidationError extends Error {
  /**
   * Creates configuration validation failure.
   *
   * @param message - safe failure explanation
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'ConfigValidationError';
  }
}

/**
 * Prepared runtime policy configuration.
 */
export type ValidatedConfig = Readonly<{
  /**
   * Whether config requests recursive authority.
   */
  recursiveChildren: boolean;
  /**
   * Built-ins followed by namespaced plugins.
   */
  registeredPolicies: readonly RuntimePolicyDefinition[];
  /**
   * Effective severity values for every registered policy.
   */
  policySeverities: Readonly<Record<string, PolicySeverity>>;
  /**
   * Runtime-parsed policy option outputs.
   */
  policyOptions: ReadonlyMap<string, unknown>;
}>;

/**
 * Allowed lifecycle trigger values.
 */
const POLICY_TRIGGERS: ReadonlySet<string> = new Set([
  'pre-forward',
  'post-commit',
  'manual-push',
  'direct-check',
  'direct-fix',
]);
/**
 * Allowed severity values.
 */
const POLICY_SEVERITIES: ReadonlySet<string> = new Set([
  'off',
  'warn',
  'error',
]);
/**
 * Allowed top-level keys.
 */
const CONFIG_KEYS: ReadonlySet<string> = new Set([
  'plugins',
  'policies',
  'trust',
]);

/**
 * Asserts ordinary non-array object.
 *
 * @param value - unknown candidate
 *
 * @returns Nothing after narrowing candidate
 *
 * @throws ConfigValidationError when candidate is not an object record
 */
function assertRecord(value: unknown,): asserts value is Record<string, unknown> {
  if (((typeof value) !== 'object') || (value === null)
    || Array.isArray(value,))
    throw new ConfigValidationError('Configuration value must be an object.',);
}

/**
 * Checks one ASCII kebab identifier character.
 *
 * @param character - one UTF-16 code unit
 *
 * @returns whether character is lowercase ASCII, decimal, or hyphen
 */
function isKebabCharacter(character: string,): boolean {
  return ((character >= 'a') && (character <= 'z'))
    || ((character >= '0') && (character <= '9'))
    || (character === '-');
}

/**
 * Validates kebab-case identifier without syntax-boundary regular expressions.
 *
 * @param value - unknown identifier
 *
 * @returns whether value is non-empty lower kebab case
 */
function isKebabIdentifier(value: unknown,): value is string {
  if (((typeof value) !== 'string') || (value.length === 0)
    || value.startsWith('-',)
    || value.endsWith('-',)
    || value.includes('/',))
    return false;
  for (let index = 0; index < value.length; index += 1) {
    if ((!isKebabCharacter(value.charAt(index,),))
      || ((value.charAt(index,) === '-') && (value.charAt(index - 1,) === '-')))
      return false;
  }
  return true;
}

/**
 * Validates known string as kebab identifier.
 *
 * @param value - string candidate
 *
 * @returns whether value is valid lower kebab case
 */
function isKebabString(value: string,): boolean {
  return isKebabIdentifier(value,);
}

/**
 * Checks policy severity.
 *
 * @param value - unknown candidate
 *
 * @returns whether value is supported severity
 */
function isPolicySeverity(value: unknown,): value is PolicySeverity {
  return ((typeof value) === 'string') && POLICY_SEVERITIES.has(value,);
}

/**
 * Checks lifecycle trigger.
 *
 * @param value - unknown candidate
 *
 * @returns whether value is supported trigger
 */
function isPolicyTrigger(value: unknown,): value is PolicyTrigger {
  return ((typeof value) === 'string') && POLICY_TRIGGERS.has(value,);
}

/**
 * Checks unknown array without leaking `any` from Array.isArray.
 *
 * @param value - unknown candidate
 *
 * @returns whether candidate is an array of unknown values
 */
function isUnknownArray(value: unknown,): value is readonly unknown[] {
  return Array.isArray(value,);
}

/**
 * Asserts unknown array without leaking `any` from Array.isArray.
 *
 * @param value - unknown candidate
 *
 * @returns Nothing after narrowing candidate
 */
function assertUnknownArray(value: unknown,): asserts value is readonly unknown[] {
  if (!isUnknownArray(value,))
    throw new ConfigValidationError('Expected an array.',);
}

/**
 * Asserts policy check callback.
 *
 * @param value - unknown callback
 *
 * @returns Nothing after narrowing callback
 */
function assertPolicyCheck(
  value: unknown,
): asserts value is RuntimePolicyDefinition['check'] {
  if ((typeof value) !== 'function')
    throw new ConfigValidationError('Policy must provide a check function.',);
}

/**
 * Asserts Valibot-compatible schema runtime protocol.
 *
 * @param value - unknown options schema
 *
 * @returns Nothing after narrowing schema
 *
 * @throws ConfigValidationError when protocol is unavailable
 */
function assertSchema(value: unknown,): asserts value is v.GenericSchema<unknown, unknown> {
  assertRecord(value,);
  if ((!('~run' in value)) || ((typeof value['~run']) !== 'function'))
    throw new ConfigValidationError('Policy options must use a Valibot schema.',);
}

/**
 * Validates policy declaration and assigns effective ID.
 *
 * @param value - unknown policy declaration
 *
 * @param effectiveId - complete engine policy ID
 *
 * @returns copied runtime policy definition
 */
function validatePolicy({
  value,
  effectiveId,
}: Readonly<{
  value: unknown;
  effectiveId: string;
}>,): RuntimePolicyDefinition {
  assertRecord(value,);
  if (!isKebabIdentifier(value.name,))
    throw new ConfigValidationError(`Policy ${effectiveId} has an invalid name.`,);
  if ((typeof value.warnSafe) !== 'boolean')
    throw new ConfigValidationError(`Policy ${effectiveId} must declare warnSafe.`,);
  if (!isPolicySeverity(value.defaultSeverity,))
    throw new ConfigValidationError(`Policy ${effectiveId} has an invalid default severity.`,);
  assertUnknownArray(value.triggers,);
  if (!value.triggers
    .every(isPolicyTrigger,))
    throw new ConfigValidationError(`Policy ${effectiveId} has invalid triggers.`,);
  assertPolicyCheck(value.check,);
  if (value.options !== undefined)
    assertSchema(value.options,);

  return {
    name: effectiveId,
    defaultSeverity: value.defaultSeverity,
    warnSafe: value.warnSafe,
    triggers: value.triggers
      .filter(isPolicyTrigger,),
    ...(value.options === undefined ? {} : { options: value.options, }),
    check: value.check,
  };
}

/**
 * Mutable view at Valibot callback boundary.
 */
type MutablePolicySchema = {
  -readonly [Key in keyof v.GenericSchema]: v.GenericSchema[Key]
};

/**
 * Invokes plugin-defined schema at mutable validation boundary.
 *
 * @param optionsSchema - plugin-defined Valibot schema
 *
 * @param rawOptions - unvalidated plugin option value
 *
 * @mutates optionsSchema - plugin callback may mutate retained schema state
 *
 * @mutates rawOptions - plugin callback may mutate supplied option value
 *
 * @returns Valibot safe parse result
 *
 * @example
 * ```ts
 * parsePolicyOptions({ optionsSchema: v.unknown(), rawOptions: 'value' });
 * ```
 */
function parsePolicyOptions({
  optionsSchema,
  rawOptions,
}: {
  optionsSchema: MutablePolicySchema;
  rawOptions: unknown;
},): v.SafeParseResult<MutablePolicySchema> {
  return v.safeParse(
    optionsSchema,
    rawOptions,
  );
}

/**
 * Parses one policy setting and options through declared schema.
 *
 * @param policyName - effective registered policy ID
 *
 * @param defaultSeverity - severity used when setting is absent
 *
 * @param optionsSchema - optional plugin-defined Valibot schema
 *
 * @param setting - explicit setting or declaration default
 *
 * @mutates optionsSchema - plugin-defined Valibot callback may mutate retained schema state
 *
 * @mutates setting - plugin-defined Valibot callback may mutate supplied option value
 *
 * @returns active severity and parsed options
 */
function parsePolicySetting({
  policyName,
  defaultSeverity,
  optionsSchema,
  setting,
}: {
  policyName: string;
  defaultSeverity: PolicySeverity;
  optionsSchema?: MutablePolicySchema;
  setting: unknown;
},): Readonly<{
  severity: PolicySeverity;
  options: unknown
}> {
  /**
   * Effective explicit setting or declaration default.
   */
  const effectiveSetting = setting ?? defaultSeverity;
  /**
   * Whether setting uses severity-plus-options tuple.
   */
  const tupleSetting = isUnknownArray(effectiveSetting,);
  /**
   * Safely narrowed tuple values.
   */
  const settingValues: readonly unknown[] = tupleSetting ? effectiveSetting : [];
  /**
   * Candidate severity value.
   */
  const severity: unknown = tupleSetting ? settingValues[0] : effectiveSetting;
  /**
   * Candidate policy options.
   */
  const rawOptions: unknown = tupleSetting ? settingValues[1] : undefined;
  if (!isPolicySeverity(severity,))
    throw new ConfigValidationError(`Policy ${policyName} has an invalid severity.`,);
  if (tupleSetting && (settingValues.length !== 2))
    throw new ConfigValidationError(`Policy ${policyName} setting tuple must contain severity and options.`,);
  if (optionsSchema === undefined) {
    if (tupleSetting)
      throw new ConfigValidationError(`Policy ${policyName} does not accept options.`,);
    return {
      severity,
      options: undefined,
    };
  }
  /**
   * Valibot runtime options result.
   */
  const parsed = parsePolicyOptions({
    optionsSchema,
    rawOptions,
  },);
  if (!parsed.success)
    throw new ConfigValidationError(`Policy ${policyName} options failed Valibot validation.`,);
  return {
    severity,
    options: parsed.output,
  };
}

/**
 * Validates imported default export and prepares policy runtime values.
 *
 * @param value - imported default export
 *
 * @mutates value - plugin-defined Valibot schemas may mutate their schema or option state
 *
 * @returns validated config plus ordered policy registry and parsed settings
 *
 * @example
 * ```ts
 * validateConfig({ plugins: {}, policies: {} });
 * ```
 */
export function validateConfig(value: unknown,): ValidatedConfig {
  assertRecord(value,);
  /**
   * First unsupported top-level key.
   */
  const unknownKey = Object.keys(value,)
    .find(function isUnknownConfigKey(key,) {
    return !CONFIG_KEYS.has(key,);
  },);
  if (unknownKey !== undefined)
    throw new ConfigValidationError(`Unknown configuration key: ${unknownKey}`,);

  /**
   * Declared plugin namespace map.
   */
  const pluginsValue = value.plugins ?? {};
  assertRecord(pluginsValue,);
  /**
   * Namespaced plugin policies in config insertion and declaration order.
   */
  const pluginPolicies = Object.entries(pluginsValue,)
    .flatMap(function validatePlugin([
    namespace,
    pluginValue,
  ],) {
    if (!isKebabString(namespace,))
      throw new ConfigValidationError(`Invalid plugin namespace: ${namespace}`,);
    assertRecord(pluginValue,);
    if (!isKebabIdentifier(pluginValue.name,))
      throw new ConfigValidationError(`Plugin ${namespace} has an invalid name.`,);
    if (!isUnknownArray(pluginValue.policies,))
      throw new ConfigValidationError(`Plugin ${namespace} policies must be an array.`,);
    /**
     * Local policy names used for duplicate rejection.
     */
    const localNames = new Set<string>();
    return pluginValue.policies
      .map(function validatePluginPolicy(policyValue,) {
      assertRecord(policyValue,);
      if ((!isKebabIdentifier(policyValue.name,)) || localNames.has(policyValue.name,))
        throw new ConfigValidationError(`Plugin ${namespace} has an invalid or duplicate policy name.`,);
      localNames.add(policyValue.name,);
      return validatePolicy({
        value: policyValue,
        effectiveId: `${namespace}/${policyValue.name}`,
      },);
    },);
  },);
  /**
   * Fixed built-ins followed by namespaced plugin policies.
   */
  const registeredPolicies: readonly RuntimePolicyDefinition[] = [
    ...BUILT_IN_POLICIES,
    ...pluginPolicies,
  ];

  /**
   * Explicit settings map or empty defaults.
   */
  const settingsValue = value.policies ?? {};
  assertRecord(settingsValue,);
  /**
   * Every effective policy ID.
   */
  const knownIds = new Set(registeredPolicies.map(function policyId(policy,) {
    return policy.name;
  },),);
  /**
   * First setting without registered policy.
   */
  const unknownPolicyId = Object.keys(settingsValue,)
    .find(function isUnknownPolicy(policyId,) {
    return !knownIds.has(policyId,);
  },);
  if (unknownPolicyId !== undefined)
    throw new ConfigValidationError(`Unknown policy ID: ${unknownPolicyId}`,);

  /**
   * Parsed option outputs by effective ID.
   */
  const policyOptions = new Map<string, unknown>();
  /**
   * Effective severity map prepared alongside options.
   */
  const policySeverities: Record<string, PolicySeverity> = {};
  for (const policy of registeredPolicies) {
    /**
     * Parsed setting for current policy.
     */
    const parsed = parsePolicySetting({
      policyName: policy.name,
      defaultSeverity: policy.defaultSeverity,
      ...(policy.options === undefined
        ? {}
        : { optionsSchema: policy.options, }),
      setting: settingsValue[policy.name],
    },);
    policyOptions.set(
      policy.name,
      parsed.options,
    );
    policySeverities[policy.name] = parsed.severity;
  }

  if (value.trust !== undefined) {
    assertRecord(value.trust,);
    /**
     * Keys present in trust declaration.
     */
    const trustKeys = Object.keys(value.trust,);
    if (trustKeys.some(function isUnknownTrustKey(key,) {
      return key !== 'children';
    },)
      || ((value.trust
        .children
        !== undefined) && ((typeof value.trust
          .children) !== 'boolean')))
      throw new ConfigValidationError('Configuration trust declaration is invalid.',);
  }

  return {
    recursiveChildren: (value.trust !== undefined) && (value.trust
      .children
      === true),
    registeredPolicies,
    policySeverities,
    policyOptions,
  };
}

/**
 JSON Schema validation for the config store.
 
 Wraps ajv in [draft-2020-12](https://json-schema.org/draft-2020-12/release-notes)
 mode with `allErrors` and `useDefaults` on,
 exactly as upstream `conf` 15.1.0 configures it,
 and surfaces violations as {@link SchemaViolationError} with upstream's
 message text.
 
 @module
 */

import {
  Ajv2020 as Ajv,
  type ValidateFunction as AjvValidateFunction,
} from 'ajv/dist/2020.js';
import ajvFormatsPlugin from 'ajv-formats';

import {
  InvalidSchemaError,
  RootSchemaPropertiesError,
  SchemaViolationError,
} from './errors.ts';
import type { ValueSchema, } from './options.ts';
import type { PreparedOptions, } from './prepare-options.ts';

//region Types

/**
 Compiled validation surface for one store.
 
 @example
 ```ts
 const validator = createSchemaValidator(options);
 validator.validate({ theme: 'dark', });
 ```
 */
export type SchemaValidator = {
  /**
   Validates one store object in place,
   applying schema defaults,
   and throws on violations.
   */
  readonly validate: (data: unknown,) => void;
  /**
   Defaults declared by the schema's property definitions,
   keyed by top-level property name.
   */
  readonly schemaDefaults: Readonly<Record<string, unknown>>;
};

//endregion Types

//region Helpers

/**
 Collects each schema property's `default` value,
 which the store exposes through `get`,
 `reset`,
 and `clear`.
 
 @param schema - Property-to-schema map from the `schema` option.
 
 @returns Defaults keyed by top-level property name.
 
 @example
 ```ts
 captureSchemaDefaults({ port: { type: 'number', default: 8080, }, });
 // => { port: 8080 }
 ```
 */
export function captureSchemaDefaults(schema: Readonly<Record<string, ValueSchema>>): Record<string, unknown> {
  /**
   Declared defaults keyed by property name.
   */
  const declaredDefaults: Record<string, unknown> = {};
  for (const [propertyName, propertySchema,] of Object.entries(schema)) {
    if (((typeof propertySchema) !== 'object') || (propertySchema === null))
      continue;
    if (!Object.hasOwn(
      propertySchema,
      'default',
    ))
      continue;
    /**
     Declared default value for this property.
     */
    const declaredDefault: unknown = propertySchema.default;
    if (declaredDefault === undefined)
      continue;
    declaredDefaults[propertyName] = declaredDefault;
  }
  return declaredDefaults;
}

/**
 Validates nothing:
 the shape used when no schema option is configured,
 since no store shape can violate an absent schema.
 
 @param data - Store object the caller asked to validate.
 
 @example
 ```ts
 validateNothing({ theme: 'dark', });
 ```
 */
function validateNothing(data: unknown,): void {
  // Intentionally empty: without a schema every store passes.
  void data;
}

//endregion Helpers

//region Factory

/**
 Compiles the store's schema into a validator,
 or a no-op validator when no schema option is configured.
 
 @param options - Prepared store options carrying `schema`,
 `rootSchema`,
 and `ajvOptions`.
 
 @returns Validator applying defaults and throwing {@link SchemaViolationError}.
 
 @throws InvalidSchemaError when `schema` is set but is not an object.
 
 @throws RootSchemaPropertiesError when `rootSchema` carries `properties`.
 
 @example
 ```ts
 const validator = createSchemaValidator(preparedOptions);
 validator.validate({ port: 8080, });
 ```
 */
export function createSchemaValidator<T extends Record<string, unknown>>(options: PreparedOptions<T>,): SchemaValidator {
  /**
   Property-to-schema map from the `schema` option.
   */
  const {schema} = options;
  /**
   Root-level JSON Schema keywords from the `rootSchema` option.
   */
  const {rootSchema} = options;
  /**
   ajv configuration overrides from the `ajvOptions` option.
   */
  const {ajvOptions} = options;
  if ((schema === undefined) && (rootSchema === undefined)
    && (ajvOptions === undefined))
    return {
      validate: validateNothing,
      schemaDefaults: {},
    };
  if ((schema !== undefined) && ((typeof schema) !== 'object'))
    throw new InvalidSchemaError();
  if ((rootSchema !== undefined) && ('properties' in rootSchema))
    throw new RootSchemaPropertiesError();
  /**
   ajv-formats entry point.
   Upstream `conf` unwraps one extra `.default` layer for its build's
   CommonJS/ESM interop (ajv-validator/ajv#2047); this repository's Node
   module resolution hands back the plugin directly,
   verified by direct invocation before adopting the plain import.
   */
  const ajvFormats = ajvFormatsPlugin;
  /**
   Validator instance configured like upstream `conf`'s:
   every error reported,
   schema defaults applied in place.
   */
  const ajv = new Ajv({
    allErrors: true,
    useDefaults: true,
    ...ajvOptions,
  },);
  ajvFormats(ajv,);
  /**
   Whole-store schema: upstream's `rootSchema` keywords around the
   `properties` map built from `schema`.
   */
  const wholeStoreSchema = {
    ...rootSchema,
    type: 'object',
    properties: schema,
  };
  /**
   Compiled validator function holding its error list.
   */
  const validator: AjvValidateFunction = ajv.compile(wholeStoreSchema,);
  return {
    validate: function validate(data: unknown,): void {
      if (validator(data,))
        return;
      /**
       Violation descriptions collected by the failed validation.
       */
      const errors = validator.errors ?? [];
      if (errors.length === 0)
        return;
      /**
       Per-property violation texts joined into upstream's message.
       */
      const violations = errors.map(
        function toViolation(error: {
          readonly instancePath: string;
          readonly message?: string;
        },): string {
          return `\`${error.instancePath
            .slice(1,)}\` ${error.message ?? ''}`;
        },
      );
      throw new SchemaViolationError({
        violations,
      },);
    },
    schemaDefaults: captureSchemaDefaults(schema ?? {},),
  };
}

//endregion Factory

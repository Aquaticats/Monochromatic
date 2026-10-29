import {
  formatJsonObject,
  isJsonObject,
  type JsonObject,
  type JsonValue,
  mergeFlatJson,
  mergeObjectDefaults,
  omitJsonKey,
  parseJsonObject,
} from '../pipeline/json.ts';
import {
  ABSENT_XML_ENTRY,
  ABSENT_XML_VALUE,
  findXmlEntryByKey,
  getXmlOptionValue,
  listXmlEntries,
  replaceOrInsertXmlEntry,
  type XmlEntry,
} from '../pipeline/xml.ts';
import {
  buildLanguageSettingsEntry,
  buildUserDefinedEntry,
} from './lsp4ij-entries.ts';

import type {
  Lsp4ijBaseServerMatch,
  Lsp4ijScopedServer,
  Lsp4ijServerSettings,
} from './lsp4ij-types.ts';

//region Match: locate the base server to derive from

/**
 Finds the base user-defined server entry matching the policy predicate.
 
 @param userDefinedXml - User-defined language-server XML content.
 
 @param match - Predicate data identifying the base server.
 
 @param excludedIds - Server ids ineligible to be the base (managed scoped servers).
 
 @returns Base server entry, or {@link ABSENT_XML_ENTRY} when none matches.
 
 @example
 ```ts
 findBaseServerEntry({ userDefinedXml, match, excludedIds: new Set() });
 ```
 */
export function findBaseServerEntry(
  {
    userDefinedXml,
    match,
    excludedIds,
  }: {
    readonly excludedIds: ReadonlySet<string>;
    readonly match: Lsp4ijBaseServerMatch;
    readonly userDefinedXml: string;
  },
): XmlEntry | typeof ABSENT_XML_ENTRY {
  /**
   First user-defined entry matching the base predicate, if any.
   */
  const found = listXmlEntries({ xml: userDefinedXml, },)
    .find(function isBaseServer(entry,): boolean {
      if (excludedIds.has(entry.key,)) return false;
      /**
       Decoded templateId option, or absent sentinel.
       */
      const templateId = getXmlOptionValue({
        block: entry.block,
        optionName: 'templateId',
      },);
      /**
       Decoded serverName option, or absent sentinel.
       */
      const serverName = getXmlOptionValue({
        block: entry.block,
        optionName: 'serverName',
      },);
      /**
       Decoded commandLine option, or absent sentinel.
       */
      const commandLine = getXmlOptionValue({
        block: entry.block,
        optionName: 'commandLine',
      },);
      return ((match.templateId !== undefined) && (templateId === match.templateId))
        || ((match.serverNameEquals !== undefined) && (serverName === match.serverNameEquals))
        || (
        (match.commandLineIncludes !== undefined)
          && (commandLine !== ABSENT_XML_VALUE)
          && (commandLine.includes(match.commandLineIncludes,))
        );
    },);
  return found ?? ABSENT_XML_ENTRY;
}

//endregion Match

//region Configuration: derive base and scoped settings objects

/**
 Parses a base server's configuration content and applies the base patch.
 
 @param block - Base language-settings entry block.
 
 @param set - Scalar overrides for the base configuration.
 
 @param arrayUnion - Array unions for the base configuration.
 
 @returns Patched configuration object.
 
 @example
 ```ts
 baseConfigObject({ block });
 ```
 */
export function baseConfigObject(
  {
    block,
    set,
    arrayUnion,
  }: {
    readonly arrayUnion?: Readonly<Record<string, readonly string[]>>;
    readonly block: string;
    readonly set?: Readonly<Record<string, JsonValue>>;
  },
): JsonObject {
  /**
   Raw configurationContent option, or absent sentinel.
   */
  const raw = getXmlOptionValue({
    block,
    optionName: 'configurationContent',
  },);
  /**
   configurationContent JSON text, defaulting to an empty object.
   */
  const content = raw === ABSENT_XML_VALUE ? '{}' : raw;
  return mergeFlatJson({
    base: parseJsonObject({
      content,
      label: 'configurationContent',
    },),
    ...(set === undefined ? {} : { set, }),
    ...(arrayUnion === undefined ? {} : { arrayUnion, }),
  },);
}

/**
 Builds the settings schema JSON, adding default properties only where absent.
 
 @param block - Base language-settings entry block.
 
 @param defaults - Schema property defaults merged under `properties`.
 
 @returns Schema JSON text shared by the base and scoped entries.
 
 @example
 ```ts
 schemaContentFor({ block });
 ```
 */
export function schemaContentFor(
  {
    block,
    defaults,
  }: {
    readonly block: string;
    readonly defaults?: Readonly<Record<string, JsonValue>>;
  },
): string {
  /**
   Raw configurationSchemaContent option, or absent sentinel.
   */
  const raw = getXmlOptionValue({
    block,
    optionName: 'configurationSchemaContent',
  },);
  /**
   Schema JSON text, defaulting to an empty properties object.
   */
  const content = raw === ABSENT_XML_VALUE ? '{"properties":{}}' : raw;
  /**
   Parsed schema object.
   */
  const schema = parseJsonObject({
    content,
    label: 'configurationSchemaContent',
  },);
  if (defaults === undefined) return formatJsonObject({ value: schema, },);
  /**
   Existing schema properties object.
   */
  const properties = isJsonObject(schema.properties,) ? schema.properties : {};
  return formatJsonObject({ value: {
    ...schema,
    properties: mergeObjectDefaults({
      base: properties,
      defaults,
    },),
  }, },);
}

/**
 Derives a scoped server's configuration from the base configuration.
 
 @param baseConfig - Patched base configuration object.
 
 @param scoped - Scoped server policy.
 
 @returns Scoped configuration object.
 
 @example
 ```ts
 scopedConfigObject({ baseConfig, scoped });
 ```
 */
export function scopedConfigObject(
  {
    baseConfig,
    scoped,
  }: {
    readonly baseConfig: Readonly<JsonObject>;
    readonly scoped: Lsp4ijScopedServer
  },
): JsonObject {
  /**
   Base config with scoped-omitted keys removed.
   */
  const omitted = (scoped.configOmitKeys ?? [])
    .reduce<JsonObject>(
    function dropKey(
      accumulator: Readonly<JsonObject>,
      key,
    ): JsonObject {
      return omitJsonKey({
        object: accumulator,
        key,
      },);
    },
    { ...baseConfig, },
    );
  return mergeFlatJson({
    base: omitted,
    ...(scoped.config
      ?.set
      === undefined ? {} : { set: scoped.config
        .set, }),
    ...(scoped.config
      ?.arrayUnion
      === undefined ? {} : { arrayUnion: scoped.config
        .arrayUnion, }),
  },);
}

//endregion Configuration

//region Render: rebuild global and user-defined settings XML

/**
 Builds updated global settings XML for the base server and scoped variants.
 
 @param languageXml - Current global settings XML.
 
 @param baseEntry - Base user-defined server entry.
 
 @param settings - Server settings policy.
 
 @returns Updated global settings XML.
 
 @throws Error when the base server has no global settings entry.
 
 @example
 ```ts
 updatedLanguageSettingsXml({ languageXml, baseEntry, settings });
 ```
 */
export function updatedLanguageSettingsXml(
  {
    languageXml,
    baseEntry,
    settings,
  }: {
    readonly baseEntry: XmlEntry;
    readonly languageXml: string;
    readonly settings: Lsp4ijServerSettings;
  },
): string {
  /**
   Base server's global settings entry, or absent sentinel.
   */
  const baseLanguageEntry = findXmlEntryByKey({
    xml: languageXml,
    key: baseEntry.key,
  },);
  if (baseLanguageEntry === ABSENT_XML_ENTRY) {
    throw new Error(`Missing language-server settings entry '${baseEntry.key}'`,);
  }
  /**
   Patched base configuration object.
   */
  const baseConfig = baseConfigObject({
    block: baseLanguageEntry.block,
    ...(settings.baseConfig
      ?.set
      === undefined ? {} : { set: settings.baseConfig
        .set, }),
    ...(settings.baseConfig
      ?.arrayUnion
      === undefined ? {} : { arrayUnion: settings.baseConfig
        .arrayUnion, }),
  },);
  /**
   Schema JSON text shared by the base and scoped entries.
   */
  const schemaContent = schemaContentFor({
    block: baseLanguageEntry.block,
    ...(settings.schemaDefaults === undefined ? {} : { defaults: settings.schemaDefaults, }),
  },);
  /**
   Global settings XML with the base server entry rebuilt.
   */
  const withBase = replaceOrInsertXmlEntry({
    xml: languageXml,
    key: baseEntry.key,
    block: buildLanguageSettingsEntry({
      serverId: baseEntry.key,
      configContent: formatJsonObject({ value: baseConfig, },),
      schemaContent,
    },),
  },);
  return settings.scopedServers
    .reduce<string>(
    function addScoped(
      xml,
      scoped,
    ): string {
      return replaceOrInsertXmlEntry({
        xml,
        key: scoped.id,
        block: buildLanguageSettingsEntry({
          serverId: scoped.id,
          configContent: formatJsonObject({ value: scopedConfigObject({
            baseConfig,
            scoped,
          },), },),
          schemaContent,
        },),
      },);
    },
    withBase,
    );
}

/**
 Builds updated user-defined server XML with each scoped server entry, via
 {@link replaceOrInsertXmlEntry} and {@link buildUserDefinedEntry}.
 
 @param userDefinedXml - Current user-defined language-server XML.
 
 @param baseEntry - Base user-defined server entry to copy options from.
 
 @param scopedServers - Scoped server policies.
 
 @returns Updated user-defined language-server XML.
 
 @example
 ```ts
 updatedUserDefinedXml({ userDefinedXml, baseEntry, scopedServers });
 ```
 */
export function updatedUserDefinedXml(
  {
    userDefinedXml,
    baseEntry,
    scopedServers,
  }: {
    readonly baseEntry: XmlEntry;
    readonly scopedServers: readonly Lsp4ijScopedServer[];
    readonly userDefinedXml: string;
  },
): string {
  return scopedServers.reduce<string>(
    function addScoped(
      xml,
      scoped,
    ): string {
      return replaceOrInsertXmlEntry({
        xml,
        key: scoped.id,
        block: buildUserDefinedEntry({
          copyOptions: scoped.copyOptions,
          fileNames: scoped.fileNames,
          languageId: scoped.languageId,
          serverId: scoped.id,
          serverName: scoped.name,
          sourceBlock: baseEntry.block,
        },),
      },);
    },
    userDefinedXml,
  );
}

//endregion Render

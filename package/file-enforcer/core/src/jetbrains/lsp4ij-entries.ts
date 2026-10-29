import {
  escapeXmlAttribute,
  xmlOptionLine,
} from '../pipeline/xml-coding.ts';
import {
  ABSENT_XML_VALUE,
  getXmlOptionValue,
} from '../pipeline/xml.ts';

//region LSP4IJ entry builders: render canonical persistent-state entries

/**
 Indentation for option lines nested inside a settings entry.
 */
const OPTION_INDENT = '              ';

/**
 Builds an XML option line copied from a source entry block, when present.
 
 @param block - Source entry block to copy from, read via {@link getXmlOptionValue}.
 
 @param optionName - Option to copy.
 
 @returns XML option line formatted via {@link xmlOptionLine}, or {@link ABSENT_XML_VALUE} when the source lacks the option.
 
 @example
 ```ts
 copiedOptionLine({ block, optionName: 'commandLine' });
 ```
 */
function copiedOptionLine(
  {
    block,
    optionName,
  }: {
    readonly block: string;
    readonly optionName: string
  },
): string | typeof ABSENT_XML_VALUE {
  /**
   Source option value, or the absent sentinel.
   */
  const value = getXmlOptionValue({
    block,
    optionName,
  },);
  if (value === ABSENT_XML_VALUE) return ABSENT_XML_VALUE;
  return xmlOptionLine({
    indent: OPTION_INDENT,
    name: optionName,
    value,
  },);
}

/**
 Builds file-pattern mapping lines binding file names to a language id.
 
 @param fileNames - File names mapped to the language id, each escaped via {@link escapeXmlAttribute}.
 
 @param languageId - LSP4IJ language id the files map to, escaped via {@link escapeXmlAttribute}.
 
 @returns XML mapping block lines.
 
 @example
 ```ts
 mappingLines({ fileNames: ['AGENTS.md'], languageId: 'markdown' });
 ```
 */
function mappingLines(
  {
    fileNames,
    languageId,
  }: {
    readonly fileNames: readonly string[];
    readonly languageId: string
  },
): string {
  /**
   One pattern line per mapped file name.
   */
  const fileNameLines = fileNames.map(function patternLine(fileName,): string {
    return `                        <option value="${escapeXmlAttribute({ value: fileName, },)}" />`;
  },);
  return [
    '              <option name="mappings">',
    `                <ServerMappingSettings languageId="${escapeXmlAttribute({ value: languageId, },)}">`,
    '                  <fileType>',
    '                    <FileTypeSettings>',
    '                      <option name="patterns">',
    ...fileNameLines,
    '                      </option>',
    '                    </FileTypeSettings>',
    '                  </fileType>',
    '                </ServerMappingSettings>',
    '              </option>',
  ].join('\n',);
}

/**
 Builds a language-server settings entry holding embedded config and schema JSON.
 
 @param serverId - LSP4IJ server id used as the entry key, escaped via {@link escapeXmlAttribute}.
 
 @param configContent - Flat language-server settings JSON text, formatted via {@link xmlOptionLine}.
 
 @param schemaContent - LSP4IJ settings schema JSON text.
 
 @returns XML entry block.
 
 @example
 ```ts
 buildLanguageSettingsEntry({ serverId, configContent, schemaContent });
 ```
 */
export function buildLanguageSettingsEntry(
  {
    configContent,
    schemaContent,
    serverId,
  }: {
    readonly configContent: string;
    readonly schemaContent: string;
    readonly serverId: string;
  },
): string {
  return [
    `        <entry key="${escapeXmlAttribute({ value: serverId, },)}">`,
    '          <value>',
    '            <LanguageServerDefinitionSettings>',
    xmlOptionLine({
      indent: OPTION_INDENT,
      name: 'configurationContent',
      value: configContent,
    },),
    xmlOptionLine({
      indent: OPTION_INDENT,
      name: 'configurationSchemaContent',
      value: schemaContent,
    },),
    xmlOptionLine({
      indent: OPTION_INDENT,
      name: 'experimentalContent',
      value: '{}',
    },),
    xmlOptionLine({
      indent: OPTION_INDENT,
      name: 'initializationOptionsContent',
      value: '{}',
    },),
    '            </LanguageServerDefinitionSettings>',
    '          </value>',
    '        </entry>',
  ].join('\n',);
}

/**
 Builds a user-defined language-server entry derived from a source server.
 Listed options are copied from the source block via {@link copiedOptionLine};
 the entry then declares its own id, name, and file-name mappings (via
 {@link mappingLines}).
 
 @param copyOptions - Option names copied from the source block, in output order.
 
 @param fileNames - File names mapped to this server.
 
 @param languageId - LSP4IJ language id the files map to.
 
 @param serverId - New server id used as the entry key and serverId option,
 escaped via {@link escapeXmlAttribute} and formatted via {@link xmlOptionLine}.
 
 @param serverName - New server display name, formatted via {@link xmlOptionLine}.
 
 @param sourceBlock - Existing server entry block to copy command and installer
 options from; entries resolving to {@link ABSENT_XML_VALUE} are dropped.
 
 @returns XML entry block.
 
 @example
 ```ts
 buildUserDefinedEntry({ copyOptions, fileNames, languageId, serverId, serverName, sourceBlock });
 ```
 */
export function buildUserDefinedEntry(
  {
    copyOptions,
    fileNames,
    languageId,
    serverId,
    serverName,
    sourceBlock,
  }: {
    readonly copyOptions: readonly string[];
    readonly fileNames: readonly string[];
    readonly languageId: string;
    readonly serverId: string;
    readonly serverName: string;
    readonly sourceBlock: string;
  },
): string {
  /**
   Option lines copied from the source block, skipping options it lacks.
   */
  const copiedLines = copyOptions
    .map(function copyOption(optionName,): string | typeof ABSENT_XML_VALUE {
      return copiedOptionLine({
        block: sourceBlock,
        optionName,
      },);
    },)
    .filter(function keepCopiedLine(line,): line is string {
      return line !== ABSENT_XML_VALUE;
    },);
  return [
    `        <entry key="${escapeXmlAttribute({ value: serverId, },)}">`,
    '          <value>',
    '            <UserDefinedLanguageServerItemSettings>',
    ...copiedLines,
    xmlOptionLine({
      indent: OPTION_INDENT,
      name: 'serverId',
      value: serverId,
    },),
    xmlOptionLine({
      indent: OPTION_INDENT,
      name: 'serverName',
      value: serverName,
    },),
    mappingLines({
      fileNames,
      languageId,
    },),
    '            </UserDefinedLanguageServerItemSettings>',
    '          </value>',
    '        </entry>',
  ].join('\n',);
}

//endregion LSP4IJ entry builders

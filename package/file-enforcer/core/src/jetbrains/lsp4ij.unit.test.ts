import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ABSENT_XML_ENTRY,
  ABSENT_XML_VALUE,
  findXmlEntryByKey,
  getXmlOptionValue,
  type Lsp4ijServerSettings,
  manageLsp4ijServerSettings,
  type XmlEntry,
} from '../../dist/final/node/index.mjs';


/**
 Minimal global settings XML holding one base server's configuration.
 */
const LANGUAGE_XML = [
  '<application>',
  '  <component name="LanguageServerSettingsState">',
  '    <state>',
  '      <map>',
  '        <entry key="srv-1">',
  '          <value>',
  '            <LanguageServerDefinitionSettings>',
  '              <option name="configurationContent" value="{&quot;harper-ls.linters.Keep&quot;: true}" />',
  '              <option name="configurationSchemaContent" value="{&quot;properties&quot;:{}}" />',
  '              <option name="experimentalContent" value="{}" />',
  '              <option name="initializationOptionsContent" value="{}" />',
  '            </LanguageServerDefinitionSettings>',
  '          </value>',
  '        </entry>',
  '      </map>',
  '    </state>',
  '  </component>',
  '</application>',
].join('\n',);

/**
 Minimal user-defined settings XML holding one base server definition.
 */
const USER_DEFINED_XML = [
  '<application>',
  '  <component name="UserDefinedLanguageServerSettingsState">',
  '    <state>',
  '      <map>',
  '        <entry key="srv-1">',
  '          <value>',
  '            <UserDefinedLanguageServerItemSettings>',
  '              <option name="commandLine" value="harper-ls" />',
  '              <option name="templateId" value="harper-ls" />',
  '              <option name="serverId" value="srv-1" />',
  '              <option name="serverName" value="Harper Language Server" />',
  '            </UserDefinedLanguageServerItemSettings>',
  '          </value>',
  '        </entry>',
  '      </map>',
  '    </state>',
  '  </component>',
  '</application>',
].join('\n',);

/**
 Policy exercising base patch, schema defaults, omit keys, and one scoped server.
 */
const SETTINGS: Lsp4ijServerSettings = {
  productPrefixes: ['IntelliJIdea',],
  baseServerMatch: {
    commandLineIncludes: 'harper-ls',
    serverNameEquals: 'Harper Language Server',
    templateId: 'harper-ls',
  },
  baseConfig: {
    set: { 'harper-ls.linters.Disabled': false, },
    arrayUnion: { 'harper-ls.excludePatterns': ['**/X.md',], },
  },
  schemaDefaults: { 'harper-ls.linters.Disabled': { type: 'boolean', default: true, }, },
  scopedServers: [
    {
      id: 'srv-scoped',
      name: 'Scoped',
      fileNames: ['X.md',],
      languageId: 'markdown',
      copyOptions: ['commandLine', 'templateId',],
      configOmitKeys: ['harper-ls.excludePatterns',],
      config: { set: { 'harper-ls.linters.ScopedOnly': false, }, },
    },
  ],
};

/**
 Sets `XDG_CONFIG_HOME` for the scope, restoring the prior value on disposal.
 
 @param configRoot - Directory to expose as the XDG config root.
 
 @returns Disposable that restores the previous environment value.
 
 @example
 ```ts
 using xdg = withXdgConfigHome('/tmp/x');
 ```
 */
function withXdgConfigHome(configRoot: string,): Disposable {
  /**
   Prior XDG config root, restored on disposal.
   */
  const previous = process.env.XDG_CONFIG_HOME;
  process.env.XDG_CONFIG_HOME = configRoot;
  return {
    [Symbol.dispose](): void {
      if (previous === undefined) delete process.env.XDG_CONFIG_HOME;
      else process.env.XDG_CONFIG_HOME = previous;
    },
  };
}

/**
 Owns a throwaway directory, removing it on async disposal.
 
 @param directory - Directory to remove on disposal.
 
 @returns Async disposable that recursively removes the directory.
 
 @example
 ```ts
 await using owned = throwawayDir(await mkdtemp(prefix));
 ```
 */
function throwawayDir(directory: string,): AsyncDisposable {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(directory, { recursive: true, force: true, },);
    },
  };
}

/**
 Narrows an entry lookup to a present entry, throwing otherwise.
 
 @param entry - Entry lookup result.
 
 @param label - Identifier surfaced when absent.
 
 @returns Present entry.
 
 @throws Error when entry is the absent sentinel.
 
 @example
 ```ts
 requireEntry(findXmlEntryByKey({ xml, key: 'a' }), 'a');
 ```
 */
function requireEntry(entry: XmlEntry | typeof ABSENT_XML_ENTRY, label: string,): XmlEntry {
  if (entry === ABSENT_XML_ENTRY) throw new Error(`expected entry '${label}' to be present`,);
  return entry;
}

/**
 Reads and parses a server entry's configuration content from settings XML.
 
 @param xml - Global settings XML.
 
 @param key - Server entry key.
 
 @returns Parsed configuration object.
 
 @throws Error when the entry or its configuration content is absent.
 
 @example
 ```ts
 configOf({ xml, key: 'srv-1' });
 ```
 */
function configOf(
  { xml, key, }: { readonly key: string; readonly xml: string; },
): Record<string, unknown> {
  /**
   Present settings entry for the key.
   */
  const entry = requireEntry(findXmlEntryByKey({ xml, key, },), key,);
  /**
   Embedded configuration content, or absent sentinel.
   */
  const content = getXmlOptionValue({ block: entry.block, optionName: 'configurationContent', },);
  if (content === ABSENT_XML_VALUE) throw new Error(`expected configurationContent for '${key}'`,);
  return JSON.parse(content,) as Record<string, unknown>;
}

await describe({
  name: '',
  // Serialized because the suite mutates the shared XDG_CONFIG_HOME environment variable.
  concurrency: 1,
  children: [
    //region manageLsp4ijServerSettings

    describe({
      name: manageLsp4ijServerSettings.name,
      children: [
        it({
          name: 'patches the base server and inserts a scoped server in both files',
          fn: async () => {
            /**
             Throwaway JetBrains config root.
             */
            const configRoot = await mkdtemp(join(tmpdir(), 'fe-lsp4ij-',),);
            await using owned = throwawayDir(configRoot,);
            /**
             Options directory of a synthetic latest IDEA install.
             */
            const optionsDirectory = join(configRoot, 'JetBrains', 'IntelliJIdea9999.9', 'options',);
            /**
             Path to the global settings file under the throwaway root.
             */
            const languagePath = join(optionsDirectory, 'LanguageServersSettings.xml',);
            /**
             Path to the user-defined settings file under the throwaway root.
             */
            const userDefinedPath = join(optionsDirectory, 'UserDefinedLanguageServerSettings.xml',);
            await mkdir(optionsDirectory, { recursive: true, },);
            await Promise.all([
              writeFile(languagePath, LANGUAGE_XML,),
              writeFile(userDefinedPath, USER_DEFINED_XML,),
            ],);
            using xdg = withXdgConfigHome(configRoot,);
            await manageLsp4ijServerSettings({
              ...SETTINGS,
              // Disposable manifest inside the throwaway root, so the sync never contends for the shared one.
              manifestPath: join(configRoot, 'staleness-manifest.json',),
            },);

            /**
             Global settings XML after the sync.
             */
            const languageXml = await readFile(languagePath, 'utf8',);
            /**
             User-defined settings XML after the sync.
             */
            const userDefinedXml = await readFile(userDefinedPath, 'utf8',);

            /**
             Base server configuration after patching.
             */
            const baseConfig = configOf({ xml: languageXml, key: 'srv-1', },);
            expect(baseConfig['harper-ls.linters.Keep'],).toBe(true,);
            expect(baseConfig['harper-ls.linters.Disabled'],).toBe(false,);
            expect(baseConfig['harper-ls.excludePatterns'],).toEqual(['**/X.md',],);

            /**
             Scoped server configuration derived from the base.
             */
            const scopedConfig = configOf({ xml: languageXml, key: 'srv-scoped', },);
            expect(scopedConfig['harper-ls.linters.Keep'],).toBe(true,);
            expect(scopedConfig['harper-ls.linters.ScopedOnly'],).toBe(false,);
            expect(scopedConfig['harper-ls.excludePatterns'],).toBeUndefined();

            /**
             Present base global settings entry, for schema inspection.
             */
            const baseLanguageEntry = requireEntry(findXmlEntryByKey({ xml: languageXml, key: 'srv-1', },), 'srv-1',);
            /**
             Embedded schema content of the base entry, or absent sentinel.
             */
            const schema = getXmlOptionValue({ block: baseLanguageEntry.block, optionName: 'configurationSchemaContent', },);
            expect((schema !== ABSENT_XML_VALUE) && (schema.includes('harper-ls.linters.Disabled',)),).toBe(true,);

            /**
             Present scoped user-defined entry.
             */
            const scopedEntry = requireEntry(findXmlEntryByKey({ xml: userDefinedXml, key: 'srv-scoped', },), 'srv-scoped',);
            expect(getXmlOptionValue({ block: scopedEntry.block, optionName: 'commandLine', },),).toBe('harper-ls',);
            expect(getXmlOptionValue({ block: scopedEntry.block, optionName: 'serverName', },),).toBe('Scoped',);
            expect(scopedEntry.block.includes('languageId="markdown"',),).toBe(true,);
            expect(scopedEntry.block.includes('<option value="X.md" />',),).toBe(true,);
            void owned;
            void xdg;
          },
        },),
      ],
    },),

    //endregion manageLsp4ijServerSettings
  ],
},);

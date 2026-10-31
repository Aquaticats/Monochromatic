//! What:
//!  Synthetic JetBrains-shaped settings documents shared by the test sidecars.
//! Why:
//!  The tests must never read the owner's real `~/.config/JetBrains` files;
//! these mirror the shapes the
//!  retired TypeScript rule's unit tests used (`lsp4ij.unit.test.ts`),
//! so the consumer's real edits are covered.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const LANGUAGE_XML = [...].join('\n');
//! ```

/// What:
/// A global settings document holding one base server's configuration.
/// Why:
/// The `LanguageServersSettings.xml` shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const LANGUAGE_XML: string;
/// ```
pub const LANGUAGE_XML: &str = r#"<application>
  <component name="LanguageServerSettingsState">
    <state>
      <map>
        <entry key="srv-1">
          <value>
            <LanguageServerDefinitionSettings>
              <option name="configurationContent" value="{&quot;harper-ls.linters.Keep&quot;: true}" />
              <option name="configurationSchemaContent" value="{&quot;properties&quot;:{}}" />
              <option name="experimentalContent" value="{}" />
              <option name="initializationOptionsContent" value="{}" />
            </LanguageServerDefinitionSettings>
          </value>
        </entry>
      </map>
    </state>
  </component>
</application>"#;

/// What:
/// A user-defined servers document holding one base server definition.
/// Why:
/// The `UserDefinedLanguageServerSettings.xml` shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const USER_DEFINED_XML: string;
/// ```
pub const USER_DEFINED_XML: &str = r#"<application>
  <component name="UserDefinedLanguageServerSettingsState">
    <state>
      <map>
        <entry key="srv-1">
          <value>
            <UserDefinedLanguageServerItemSettings>
              <option name="commandLine" value="harper-ls" />
              <option name="templateId" value="harper-ls" />
              <option name="serverId" value="srv-1" />
              <option name="serverName" value="Harper Language Server" />
            </UserDefinedLanguageServerItemSettings>
          </value>
        </entry>
      </map>
    </state>
  </component>
</application>"#;

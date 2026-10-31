//! What:
//!  Synthetic JetBrains-shaped settings files and the policy config shared by the Harper tests.
//! Why:
//!  The tests must never read the owner's real `~/.config/JetBrains` files;
//! these mirror the shapes the IDE writes
//!  (two-space indentation, embedded JSON in attributes, `&quot;` and `&#10;` references).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const LANGUAGE_XML: string; export const USER_XML: string; export const POLICY_CONFIG: string;
//! ```

/// The base server's id in both files.
pub const BASE_KEY: &str = "srv-1";

/// The scoped server's id in the policy.
pub const SCOPED_KEY: &str = "harper-ls-agents-claude-md";

/// A `LanguageServersSettings.xml` holding the base server's configuration,
/// with a user key, a user exclude pattern and a comment that must survive.
pub const LANGUAGE_XML: &str = r#"<application>
  <component name="LanguageServerSettingsState">
    <state>
      <map>
        <entry key="srv-1">
          <!-- keep this comment -->
          <value>
            <LanguageServerDefinitionSettings>
              <option name="configurationContent" value="{&#10;  &quot;harper-ls.linters.Keep&quot;: true,&#10;  &quot;harper-ls.excludePatterns&quot;: [&#10;    &quot;**/mine.md&quot;&#10;  ]&#10;}" />
              <option name="configurationSchemaContent" value="{&quot;properties&quot;:{}}" />
              <option name="experimentalContent" value="{}" />
              <option name="initializationOptionsContent" value="{}" />
            </LanguageServerDefinitionSettings>
          </value>
        </entry>
      </map>
    </state>
  </component>
</application>
"#;

/// A `UserDefinedLanguageServerSettings.xml` holding the base server's definition.
pub const USER_XML: &str = r#"<application>
  <component name="UserDefinedLanguageServerSettingsState">
    <state>
      <map>
        <entry key="srv-1">
          <value>
            <UserDefinedLanguageServerItemSettings>
              <option name="commandLine" value="harper-ls --stdio" />
              <option name="installAlreadyDone" value="true" />
              <option name="templateId" value="harper-ls" />
              <option name="serverId" value="srv-1" />
              <option name="serverName" value="Harper Language Server" />
              <option name="workingDir" value="/work" />
            </UserDefinedLanguageServerItemSettings>
          </value>
        </entry>
      </map>
    </state>
  </component>
</application>
"#;

/// A per-user config with one `jetbrains-harper` rule:
/// three base linters off,
/// two excluded files,
/// one scoped server.
pub const POLICY_CONFIG: &str = r#"{"rules": [{"kind": "jetbrains-harper", "id": "harper",
  "productPrefixes": ["IntelliJIdea", "IdeaIC"],
  "disabledLinters": ["UseTitleCase", "SplitWords", "PhrasalVerbAsCompoundNoun"],
  "excludePatterns": ["**/AGENTS.md", "**/CLAUDE.md"],
  "scopedServers": [{"id": "harper-ls-agents-claude-md", "name": "Harper Language Server (AGENTS.md and CLAUDE.md)",
    "fileNames": ["AGENTS.md", "CLAUDE.md"], "languageId": "markdown",
    "disabledLinters": ["MissingTo", "LongSentences", "OxfordComma"]}]}]}"#;

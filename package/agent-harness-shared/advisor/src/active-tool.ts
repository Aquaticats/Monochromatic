/**
 Active tool-list queries for Advisor session toggling.
 
 @module
 */

/**
 Check whether active tool names contain target tool.
 
 @param toolNames - active tool names
 
 @param targetName - target tool name
 
 @returns whether target tool is active
 
 @example
 ```typescript
 containsToolName({ toolNames: ['advisor'], targetName: 'advisor' });
 ```
 */
export function containsToolName(
  {
    toolNames,
    targetName,
  }: {
    readonly toolNames: readonly string[];
    readonly targetName: string;
  },
): boolean {
  for (const toolName of toolNames) {
    if (toolName
      === targetName)
      return true;
  }
  return false;
}

/**
 Notebook and LSP tool input shapes.
 
 Separated from `tool-inputs-extended.ts` to keep each file under the max-lines limit.
 
 @module
 */

/**
 Input shape for the `NotebookEdit` tool.
 
 @example
 ```ts
 if (event.tool_name === 'NotebookEdit') {
   const { notebook_path } = event.tool_input as NotebookEditToolInput;
 }
 ```
 */
export type NotebookEditToolInput = {
  /**
   Absolute path to the Jupyter notebook file.
   */
  notebook_path: string;

  /**
   New source content for the cell.
   */
  new_source: string;

  /**
   ID of the cell to edit.
   */
  cell_id?: string;

  /**
   Cell type.
   */
  cell_type?: 'code' | 'markdown';

  /**
   Edit operation type.
   */
  edit_mode?: 'replace' | 'insert' | 'delete';
};

/**
 LSP operation names supported by the `LSP` tool.
 */
export type LspOperation =
  | 'goToDefinition'
    | 'findReferences'
    | 'hover'
    | 'documentSymbol'
    | 'workspaceSymbol'
    | 'goToImplementation'
    | 'prepareCallHierarchy'
    | 'incomingCalls'
    | 'outgoingCalls';

/**
 Input shape for the `LSP` tool.
 
 @example
 ```ts
 if (event.tool_name === 'LSP') {
   const { operation, filePath } = event.tool_input as LspToolInput;
 }
 ```
 */
export type LspToolInput = {
  /**
   LSP operation to perform.
   */
  operation: LspOperation;

  /**
   Absolute or relative path to the file.
   */
  filePath: string;

  /**
   Line number (1-based).
   */
  line: number;

  /**
   Character offset (1-based).
   */
  character: number;
};

// Prompt Dialog Polyfill: Drop-in replacement for globalThis.prompt using dialog element

/**
 Per-call class-name overrides for the dialog, cancel button, and OK button.
 
 Any field left undefined falls back to the corresponding entry in
 {@link DEFAULT_PROMPT_CLASSES}, so existing stylesheets keyed on the
 default names continue to work without changes.
 
 @example
 ```ts
 await prompt({
   message: 'Rename file',
   classes: { dialog: 'rename-dialog', ok: 'rename-ok', },
 },);
 ```
 */
export type PromptClassNames = {
  /**
   Class applied to the `<dialog>` element.
   */
  readonly dialog?: string;
  /**
   Class applied to the Cancel button.
   */
  readonly cancel?: string;
  /**
   Class applied to the OK (submit) button.
   */
  readonly ok?: string;
};

/**
 Default class names applied to the dialog parts when no per-call
 {@link PromptClassNames} are supplied. Exported so consumers can
 target the same names in their stylesheet without duplicating literals.
 */
export const DEFAULT_PROMPT_CLASSES: Required<PromptClassNames> = {
  dialog: 'prompt-polyfill-dialog',
  cancel: 'prompt-polyfill-cancel',
  ok: 'prompt-polyfill-ok',
};

/* oxlint-disable require-await, no-restricted-syntax/no-nullish-union -- exposed as async so callers can `await` even though the work is event-driven; the `string | null` return mirrors the native `globalThis.prompt` DOM API this polyfill replaces, which returns the entered string (including `''`) on OK or `null` on every cancel path */
/**
 Creates a modern prompt dialog using the HTML dialog element.
 This serves as a polyfill for globalThis.prompt with enhanced styling capabilities.
 
 @param message - Message to display to the user
 
 @param defaultValue - Default value for the input field
 
 @param classes - Optional per-call class-name overrides; unset fields fall
   back to {@link DEFAULT_PROMPT_CLASSES}. Use when two prompts on the same
   page need distinct styling, since a global stylesheet keyed on the
   defaults cannot differentiate them.
 
 @returns Promise that resolves to the entered string when OK is clicked
   (including `''` for an empty field), or `null` when the user cancels
   via the Cancel button, the Esc key, or a backdrop click. Mirrors the
   distinction native `globalThis.prompt` makes between empty-OK and cancel.
 
 @example
 ```ts
 const name = await prompt({ message: 'What is your name?', },);
 if (name !== null) {
   console.log(`Hello, ${name}!`,);
 }
 ```
 */
export async function prompt(
  {
    message,
    defaultValue = '',
    classes,
  }: Readonly<{
    message: string;
    defaultValue?: string;
    classes?: PromptClassNames;
  }>,
): Promise<string | null> {
  /**
   Per-call classes merged with {@link DEFAULT_PROMPT_CLASSES} so unset fields stay defaulted.
   */
  const resolvedClasses = {
    ...DEFAULT_PROMPT_CLASSES,
    ...classes,
  };

  // oxlint-disable-next-line promise/avoid-new -- Required for dialog event handling
  return new Promise(function promptExecutor(resolve,) {
    /**
     Tracks whether the dialog was closed via OK (false) or any cancel path (true).
     
     Initialised to `true` so Esc, backdrop click, and any other dialog close
     default to "cancelled"; only the submit handler flips it.
     */
    const state = { cancelled: true, };

    /**
     Modal element that hosts the prompt form.
     */
    const dialog = document.createElement('dialog',);
    dialog.className = resolvedClasses.dialog;

    /**
     Inner form; uses method="dialog" so submit closes the dialog natively.
     */
    const form = document.createElement('form',);
    form.method = 'dialog';

    /**
     Heading element carrying the prompt message.
     */
    const titleElement = document.createElement('h2',);
    titleElement.textContent = message;

    /**
     Text input that captures the user's response.
     */
    const input = document.createElement('input',);
    input.type = 'text';
    input.value = defaultValue;
    input.autofocus = true;

    /**
     Wrapper that lays out the Cancel and OK buttons side by side.
     */
    const buttonContainer = document.createElement('div',);

    /**
     Cancel button; leaves `state.cancelled = true` and closes the dialog.
     */
    const cancelButton = document.createElement('button',);
    cancelButton.type = 'button';
    cancelButton.className = resolvedClasses.cancel;
    cancelButton.textContent = 'Cancel';

    /**
     OK button; submits the form to flip `state.cancelled` and close.
     */
    const okButton = document.createElement('button',);
    okButton.type = 'submit';
    okButton.className = resolvedClasses.ok;
    okButton.textContent = 'OK';

    // Assemble the dialog
    buttonContainer.append(cancelButton,);
    buttonContainer.append(okButton,);
    form.append(titleElement,);
    form.append(input,);
    form.append(buttonContainer,);
    dialog.append(form,);
    document.body
      .append(dialog,);

    // Handle form submission (OK click or Enter key). This is the only path that
    // counts as a successful prompt; `state.cancelled` flips so the close handler
    // can resolve to the entered string instead of null, even when the string is empty.
    form.addEventListener(
      'submit',
      /**
       Accepts submitted value and prevents browser navigation.
       
       @param event - Form submission event owned by browser.
       
       @mutates event - `event.preventDefault` marks submission as canceled.
       */
      function onSubmit(event,): void {
        event.preventDefault();
        state.cancelled = false;
        dialog.close();
      },
    );

    // Handle cancel button: leave state.cancelled = true and just close.
    cancelButton.addEventListener(
      'click',
      function onCancelClick() {
        dialog.close();
      },
    );

    // Handle dialog close (Esc key, programmatic close, or close after submit/cancel above).
    // `state.cancelled` decides between native-prompt's two return shapes:
    // entered string (including '') on OK, null on every cancel path.
    dialog.addEventListener(
      'close',
      function onClose() {
        /**
         Final value handed to the caller: `null` on every cancel path, entered string on OK.
         */
        const result = state.cancelled ? null : input.value;

        dialog.remove();

        resolve(result,);
      },
    );

    // Handle backdrop click: treat as cancellation, same as the Cancel button.
    dialog.addEventListener(
      'click',
      function onBackdropClick(event,) {
        // Check if click was on the backdrop (dialog element itself, not its children).
        if (event.target
          === dialog) {
          /**
           Dialog box rectangle used to distinguish backdrop clicks from content clicks.
           */
          const rect = dialog.getBoundingClientRect();
          /**
           True when the pointer was inside the visible dialog box, not on the backdrop.
           */
          const clickedInDialog = (event.clientX
            >= rect
              .left)
            && (event.clientX
              <= rect
                .right)
            && (event.clientY
              >= rect
                .top)
            && (event.clientY
              <= rect
                .bottom);

          if (!clickedInDialog)
            dialog.close();
        }
      },
    );

    // Show the dialog
    dialog.showModal();

    // Select text in input for easy replacement
    input.select();
  },);
}
/* oxlint-enable require-await, no-restricted-syntax/no-nullish-union */

// Single-key menu shortcuts (Q/E, Enter, M...) must stay out of the way while the player types.

/** True when a key event lands in a text field, dropdown or in-place editable text. */
export function isTyping(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    (target instanceof HTMLElement && target.isContentEditable)
  );
}

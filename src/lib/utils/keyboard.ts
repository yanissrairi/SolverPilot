export interface ShortcutConfig {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
}

export function matchesShortcut(event: KeyboardEvent, config: ShortcutConfig): boolean {
  const key = config.key.toLowerCase();
  const eventKey = event.key.toLowerCase();
  const eventCode = event.code.toLowerCase();

  // Check key match (allow 'Enter' to match 'enter' or 'Return')
  // We check both key and code to be safe, but usually 'key' is enough for characters
  // and 'code' is better for physical location.
  // For simple letters: key='a', code='KeyA'.
  // For Enter: key='Enter', code='Enter'.

  const keyMatch = eventKey === key || eventCode === key || eventCode === `key${key}`;
  if (!keyMatch) return false;

  // Check modifiers
  const expectCtrl = config.ctrl ?? false;
  const expectShift = config.shift ?? false;
  const expectAlt = config.alt ?? false;

  // On Mac, Meta (Command) is usually the trigger for 'Ctrl' shortcuts in web apps
  // unless specifically distinguished. For Linux/Windows, it's Ctrl.
  // We'll allow either for the 'ctrl' flag to be robust.
  const pressedCtrl = event.ctrlKey || event.metaKey;

  if (expectCtrl !== pressedCtrl) return false;
  if (expectShift !== event.shiftKey) return false;
  if (expectAlt !== event.altKey) return false;

  return true;
}

// Panel sizes store with localStorage persistence
// Note: $effect cannot be used outside component context, so we save manually in setters

function loadFromStorage(key: string, defaultValue: number): number {
  if (typeof localStorage === 'undefined') return defaultValue;
  const saved = localStorage.getItem(key);
  return saved !== null ? parseInt(saved, 10) : defaultValue;
}

function saveToStorage(key: string, value: number): void {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(key, value.toString());
  }
}

class PanelStore {
  leftWidth = $state(loadFromStorage('panel-left-width', 280));
  rightWidth = $state(loadFromStorage('panel-right-width', 300));

  setLeftWidth(width: number) {
    this.leftWidth = width;
    saveToStorage('panel-left-width', width);
  }

  setRightWidth(width: number) {
    this.rightWidth = width;
    saveToStorage('panel-right-width', width);
  }
}

export const panelStore = new PanelStore();

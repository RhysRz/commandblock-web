export function createSettingsStore() {
  let value = { model: 'deepseek-v4-flash', baseUrl: 'https://api.deepseek.com', apiKey: '' };
  return {
    get: () => ({ ...value }),
    set: (next) => { value = { ...value, ...next }; },
    clear: () => { value.apiKey = ''; },
  };
}

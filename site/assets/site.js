// Share Stucco's saved choice and pre-paint theme initialization.
const themeSwitch = document.querySelector('.theme-switch');
if (themeSwitch) {
  const preference = window.matchMedia('(prefers-color-scheme: dark)');
  const storageKey = themeSwitch.dataset.themeStorageKey;
  const root = document.documentElement;
  const updateSwitch = () => {
    const dark = root.dataset.theme ? root.dataset.theme === 'dark' : preference.matches;
    themeSwitch.setAttribute('aria-checked', String(dark));
  };
  themeSwitch.addEventListener('click', () => {
    const theme = themeSwitch.getAttribute('aria-checked') === 'true' ? 'light' : 'dark';
    root.dataset.theme = theme;
    try { localStorage.setItem(storageKey, theme); } catch { /* Still switch this page. */ }
    updateSwitch();
  });
  preference.addEventListener('change', updateSwitch);
  window.addEventListener('storage', event => {
    if (event.key !== storageKey && event.key !== null) return;
    if (event.newValue === 'light' || event.newValue === 'dark') root.dataset.theme = event.newValue;
    else delete root.dataset.theme;
    updateSwitch();
  });
  updateSwitch();
  themeSwitch.hidden = false;
}

// Code remains selectable and navigation works when scripts are unavailable.
const status = document.createElement('span');
status.className = 'visually-hidden';
status.setAttribute('role', 'status');
document.body.append(status);

for (const block of document.querySelectorAll('pre')) {
  const code = block.querySelector('code');
  if (!code) continue;
  const frame = document.createElement('div');
  frame.className = 'code-frame';
  block.before(frame);
  frame.append(block);
  const button = document.createElement('button');
  button.type = 'button';
  button.className = 'copy-code';
  button.textContent = 'Copy';
  button.setAttribute('aria-label', 'Copy code');
  frame.append(button);
  button.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(code.textContent);
      button.textContent = 'Copied';
      status.textContent = 'Code copied to clipboard.';
    } catch {
      const range = document.createRange();
      range.selectNodeContents(code);
      const selection = window.getSelection();
      selection.removeAllRanges();
      selection.addRange(range);
      status.textContent = 'Code selected. Use your browser’s copy command.';
    }
    window.setTimeout(() => { button.textContent = 'Copy'; }, 1800);
  });
}

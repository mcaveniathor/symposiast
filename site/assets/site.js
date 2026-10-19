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

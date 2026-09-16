(() => {
  const root = document.getElementById('rullst-admin-ai');
  if (!root || root.dataset.ready) return;
  root.dataset.ready = 'true';
  const form = root.querySelector('form');
  const input = form.querySelector('textarea');
  const submit = form.querySelector('[type="submit"]');
  const csrf = form.querySelector('[name="_token"]');
  const messages = root.querySelector('.ai-messages');
  const status = root.querySelector('.ai-status');
  root.querySelectorAll('[data-prompt]').forEach(button => button.addEventListener('click', () => {
    input.value = button.dataset.prompt;
    input.focus();
  }));
  const bubble = (text, type) => {
    const item = document.createElement('div');
    item.className = 'ai-message ai-' + type;
    item.textContent = text;
    messages.append(item);
    while (messages.children.length > 40) messages.firstElementChild.remove();
    messages.scrollTop = messages.scrollHeight;
    return item;
  };
  form.addEventListener('submit', async event => {
    event.preventDefault();
    const message = input.value.trim();
    if (!message || submit.disabled) return;
    bubble(message, 'user');
    input.value = '';
    submit.disabled = true;
    status.textContent = 'Thinking…';
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 35000);
    try {
      const response = await fetch(form.action, {
        method: 'POST', credentials: 'same-origin', signal: controller.signal,
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
          'X-Rullst-AI': '1',
          'X-CSRF-Token': csrf.value
        },
        body: new URLSearchParams({ message, _token: csrf.value })
      });
      if (!response.ok) {
        const errors = {
          400: 'Please rephrase your question.',
          401: 'Your authentication expired. Reopen the panel to sign in.',
          403: 'The security token expired. Reload this page and try again.',
          429: 'The assistant is busy. Please retry in a minute.'
        };
        bubble(errors[response.status] || 'AI is temporarily unavailable. Please try again.', 'assistant');
      } else {
        // Only our authenticated endpoint's server-sanitized fragment is HTML.
        const html = await response.text();
        bubble('', 'assistant').innerHTML = html;
      }
    } catch {
      bubble('Could not get a reply. Please retry.', 'assistant');
    } finally {
      clearTimeout(timeout);
      submit.disabled = false;
      status.textContent = '';
      messages.scrollTop = messages.scrollHeight;
      input.focus();
    }
  });
})();

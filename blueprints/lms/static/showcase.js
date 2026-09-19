(() => {
  try { localStorage.removeItem('htmx-history-cache'); } catch (_) {}
  if (window.htmx) htmx.config.historyCacheSize = 0;
  const status = document.getElementById('privacy-choice-status');
  const say = text => { if (status) status.textContent = text; };
  // Migrate existing installations without opting new visitors into storage.
  if ('serviceWorker' in navigator) navigator.serviceWorker.getRegistration('/').then(reg => {
    if (reg && reg.active && new URL(reg.active.scriptURL).pathname === '/sw.js') return reg.update();
  }).catch(() => {});
  document.getElementById('enable-offline')?.addEventListener('click', async () => {
    try { await navigator.serviceWorker.register('/sw.js'); say('Offline assets enabled. Pages and account data are never cached.'); }
    catch (_) { say('Offline assets are unavailable in this browser.'); }
  });
  document.getElementById('reset-privacy')?.addEventListener('click', async () => {
    try {
      if ('serviceWorker' in navigator) {
        const reg = await navigator.serviceWorker.getRegistration('/');
        if (reg && reg.active && new URL(reg.active.scriptURL).pathname === '/sw.js') await reg.unregister();
      }
      if ('caches' in window) await Promise.all((await caches.keys()).filter(name => name.startsWith('rullst-shell-')).map(name => caches.delete(name)));
      document.querySelectorAll('input[name="cloud_ai"]').forEach(input => { input.checked = false; });
      say('Optional storage disabled and Rullst cached assets removed. Reload other open LMS tabs to finish applying this choice.');
    } catch (_) { say('Your browser could not clear storage. Use its site settings to remove LMS site data.'); }
  });
  document.querySelectorAll('[data-video-src]').forEach(container => {
    container.querySelector('button')?.addEventListener('click', () => {
      const src = new URL(container.dataset.videoSrc, location.origin);
      if (!['www.youtube.com', 'www.youtube-nocookie.com'].includes(src.hostname) || !/^\/embed\/[a-zA-Z0-9_-]+$/.test(src.pathname)) return;
      src.hostname = 'www.youtube-nocookie.com';
      const frame = document.createElement('iframe');
      frame.src = src.href; frame.title = container.dataset.videoTitle || 'Lesson video';
      frame.allow = 'encrypted-media; picture-in-picture'; frame.allowFullscreen = true; frame.referrerPolicy = 'no-referrer';
      const stop = document.createElement('button'); stop.type = 'button'; stop.textContent = 'Unload video'; stop.addEventListener('click', () => location.reload());
      container.replaceChildren(frame, stop);
    });
  });
  const viewport = () => document.documentElement.style.setProperty('--lms-viewport-height', `${window.visualViewport?.height || innerHeight}px`);
  window.visualViewport?.addEventListener('resize', viewport); viewport();
})();

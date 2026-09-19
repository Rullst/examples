(() => {
  if (window.RullstCopilot) return;
  const states = new WeakMap();
  const stateFor = drawer => {
    if (!states.has(drawer)) states.set(drawer, { locked: false, siblings: [], focus: null, overflow: '' });
    return states.get(drawer);
  };
  const isOpen = drawer => drawer.style.display !== 'none';
  const syncViewport = drawer => {
    const viewport = window.visualViewport;
    drawer.style.setProperty('--copilot-height', `${viewport?.height || innerHeight}px`);
    drawer.style.setProperty('--copilot-width', `${viewport?.width || innerWidth}px`);
    drawer.style.setProperty('--copilot-top', `${viewport?.offsetTop || 0}px`);
    drawer.style.setProperty('--copilot-left', `${viewport?.offsetLeft || 0}px`);
    drawer.style.setProperty('--copilot-bottom', `${viewport ? Math.max(0, innerHeight - viewport.height - viewport.offsetTop) : 0}px`);
    drawer.classList.toggle('copilot-short', (viewport?.height || innerHeight) < 500);
  };
  function syncModal(drawer) {
    const state = stateFor(drawer);
    const modal = isOpen(drawer) && (drawer.classList.contains('copilot-expanded') || innerWidth <= 760);
    if (modal && !state.locked) {
      state.overflow = document.body.style.overflow;
      document.body.style.overflow = 'hidden';
      // Preserve pre-existing inert states and support widgets nested in page layouts.
      let node = drawer;
      while (node && node !== document.body) {
        for (const sibling of node.parentElement?.children || []) {
          if (sibling !== node && (!drawer.dataset.copilotBackdrop || sibling.id !== drawer.dataset.copilotBackdrop) && !['SCRIPT', 'STYLE'].includes(sibling.tagName)) {
            state.siblings.push([sibling, sibling.inert]);
            sibling.inert = true;
          }
        }
        node = node.parentElement;
      }
      state.locked = true;
    } else if (!modal && state.locked) {
      for (const [node, inert] of state.siblings) node.inert = inert;
      state.siblings = [];
      document.body.style.overflow = state.overflow;
      state.locked = false;
    }
    if (modal) drawer.setAttribute('aria-modal', 'true');
    else drawer.removeAttribute('aria-modal');
  }
  function expand(drawer, expanded) {
    drawer.classList.toggle('copilot-expanded', expanded);
    const button = drawer.querySelector('[data-copilot-expand]');
    button.textContent = expanded ? '\u2750 Restore' : '\u26F6 Expand';
    button.setAttribute('aria-pressed', String(expanded));
    button.setAttribute('aria-label', expanded ? 'Restore compact chat' : 'Expand chat to full screen');
    button.title = button.getAttribute('aria-label');
    syncViewport(drawer);
    syncModal(drawer);
  }
  window.RullstCopilot = {
    toggle(id) {
      const drawer = document.getElementById(id);
      if (!drawer) return;
      const state = stateFor(drawer);
      const launcher = document.getElementById(drawer.dataset.copilotLauncher);
      const backdrop = document.getElementById(drawer.dataset.copilotBackdrop);
      if (isOpen(drawer)) {
        drawer.style.display = 'none';
        expand(drawer, false);
        backdrop?.classList.remove('open');
        if (launcher) { launcher.style.display = 'flex'; launcher.setAttribute('aria-expanded', 'false'); }
        (state.focus?.isConnected ? state.focus : launcher)?.focus({ preventScroll: true });
      } else {
        state.focus = document.activeElement;
        drawer.style.display = 'flex';
        if (launcher) { launcher.style.display = 'none'; launcher.setAttribute('aria-expanded', 'true'); }
        if (innerWidth <= 760) backdrop?.classList.add('open');
        syncViewport(drawer);
        syncModal(drawer);
        // Opening on a phone must not automatically bring up the keyboard.
        const focus = innerWidth <= 760 ? drawer.querySelector('[data-copilot-expand]') : drawer.querySelector('input[name="message"]');
        focus?.focus({ preventScroll: true });
      }
    }
  };
  function init() {
    document.querySelectorAll('[data-copilot]').forEach(drawer => {
      syncViewport(drawer);
      const launcher = document.getElementById(drawer.dataset.copilotLauncher);
      launcher?.setAttribute('aria-controls', drawer.id);
      launcher?.setAttribute('aria-expanded', 'false');
      drawer.querySelector('[data-copilot-expand]')?.addEventListener('click', () => expand(drawer, !drawer.classList.contains('copilot-expanded')));
      drawer.addEventListener('keydown', event => {
        if (event.key === 'Escape' && drawer.classList.contains('copilot-expanded')) {
          event.preventDefault(); event.stopPropagation();
          expand(drawer, false);
          drawer.querySelector('[data-copilot-expand]').focus({ preventScroll: true });
        } else if (event.key === 'Tab' && drawer.getAttribute('aria-modal') === 'true') {
          const controls = [...drawer.querySelectorAll('button,a[href],input,textarea,[tabindex="0"]')].filter(node => !node.disabled && node.getClientRects().length);
          const first = controls[0], last = controls.at(-1);
          if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus(); }
          else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus(); }
        }
      });
    });
  }
  const resize = () => document.querySelectorAll('[data-copilot]').forEach(drawer => { syncViewport(drawer); syncModal(drawer); });
  window.addEventListener('resize', resize);
  window.visualViewport?.addEventListener('resize', resize);
  window.visualViewport?.addEventListener('scroll', resize);
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', init, { once: true });
  else init();
})();

(() => {
  function syncViewport() {
    const viewport = window.visualViewport;
    document.documentElement.style.setProperty('--portfolio-viewport-height', `${viewport ? viewport.height : innerHeight}px`);
    document.documentElement.style.setProperty('--nexus-viewport-height', `${viewport ? viewport.height : innerHeight}px`);
    document.documentElement.style.setProperty('--portfolio-viewport-bottom', `${viewport ? Math.max(0, innerHeight - viewport.height - viewport.offsetTop) : 0}px`);
  }
  syncViewport();
  window.addEventListener('resize', syncViewport);
  window.visualViewport?.addEventListener('resize', syncViewport);
  window.visualViewport?.addEventListener('scroll', syncViewport);
  document.getElementById('ai-crab-launcher')?.addEventListener('keydown', event => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      window.toggleAiDrawer();
    }
  });
  // A restored history entry must not silently restore permission to send to AI.
  window.addEventListener('pageshow', () => {
    document.querySelectorAll('[name="cloud_ai"]').forEach(input => { input.checked = false; });
  });
})();

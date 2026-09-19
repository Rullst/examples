(() => {
    try { localStorage.removeItem('htmx-history-cache'); } catch (_) { /* Storage can be disabled. */ }
    const sidebar = document.getElementById('showcase-sidebar');
    const toggle = document.getElementById('showcase-nav-toggle');
    const close = document.getElementById('showcase-nav-close');
    const backdrop = document.getElementById('showcase-nav-backdrop');
    if (!sidebar || !toggle || !close || !backdrop) return;
    const mobile = matchMedia('(max-width: 1000px)');
    let previousOverflow = '';
    const inertState = new Map();
    function setOpen(open, returnFocus = false) {
        open = open && mobile.matches;
        const wasOpen = sidebar.classList.contains('is-open');
        sidebar.classList.toggle('is-open', open);
        sidebar.inert = mobile.matches && !open;
        toggle.setAttribute('aria-expanded', String(open));
        backdrop.hidden = !open;
        if (open) {
            sidebar.setAttribute('role', 'dialog');
            sidebar.setAttribute('aria-modal', 'true');
            if (!wasOpen) {
                previousOverflow = document.body.style.overflow;
                for (const child of document.body.children) {
                    if (child === sidebar || child === backdrop || child.tagName === 'SCRIPT') continue;
                    inertState.set(child, child.inert);
                    child.inert = true;
                }
                document.body.style.overflow = 'hidden';
                close.focus();
            }
        } else {
            sidebar.removeAttribute('role');
            sidebar.removeAttribute('aria-modal');
            for (const [element, wasInert] of inertState) element.inert = wasInert;
            inertState.clear();
            if (wasOpen) document.body.style.overflow = previousOverflow;
            if (returnFocus) toggle.focus();
        }
    }
    document.documentElement.setAttribute('data-showcase-nav-ready', 'true');
    setOpen(false);
    toggle.addEventListener('click', () => setOpen(true));
    close.addEventListener('click', () => setOpen(false, true));
    backdrop.addEventListener('click', () => setOpen(false, true));
    sidebar.addEventListener('click', event => {
        if (event.target.closest('a') && mobile.matches) setOpen(false, true);
    });
    document.addEventListener('keydown', event => {
        if (!sidebar.classList.contains('is-open')) return;
        if (event.key === 'Escape') setOpen(false, true);
        if (event.key === 'Tab') {
            const items = [...sidebar.querySelectorAll('a,button')].filter(el => el.getClientRects().length);
            const first = items[0], last = items[items.length - 1];
            if (event.shiftKey && document.activeElement === first) {
                event.preventDefault(); last.focus();
            } else if (!event.shiftKey && document.activeElement === last) {
                event.preventDefault(); first.focus();
            }
        }
    });
    mobile.addEventListener('change', () => setOpen(false));
})();

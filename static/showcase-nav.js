(() => {
    try { localStorage.removeItem('htmx-history-cache'); } catch (_) { /* Storage can be disabled. */ }
    const menus = [...document.querySelectorAll('.showcase-menu')];
    const closeMenus = (except) => menus.forEach((menu) => {
        if (menu !== except) menu.open = false;
    });

    menus.forEach((menu) => {
        menu.addEventListener('toggle', () => {
            if (menu.open) closeMenus(menu);
        });
        menu.addEventListener('click', (event) => {
            if (event.target.closest('a')) menu.open = false;
        });
    });
    document.addEventListener('click', (event) => {
        if (!event.target.closest('.showcase-menu')) closeMenus();
    });
    document.addEventListener('keydown', (event) => {
        if (event.key !== 'Escape') return;
        const openMenu = menus.find((menu) => menu.open);
        if (openMenu) {
            openMenu.open = false;
            openMenu.querySelector('summary').focus();
        }
    });
    document.addEventListener('focusin', (event) => {
        closeMenus(event.target.closest('.showcase-menu'));
    });
})();

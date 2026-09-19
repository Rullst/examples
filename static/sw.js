// Cache only public static assets, never pages, credentials or user content.
const CACHE_NAME = 'rullst-shell-v2';
const STATIC_ASSETS = new Set([
  '/manifest.webmanifest',
  '/static/icon-192.png',
  '/static/icon-512.png',
  '/static/crab.png',
  '/static/htmx.js',
  '/static/tailwind.js'
]);

self.addEventListener('install', (event) => {
  event.waitUntil(self.skipWaiting());
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((names) => Promise.all(
      names.filter((name) => name.startsWith('rullst-shell-') && name !== CACHE_NAME)
        .map((name) => caches.delete(name))
    )).then(() => self.clients.claim())
  );
});

self.addEventListener('fetch', (event) => {
  const request = event.request;
  const url = new URL(request.url);
  if (request.method !== 'GET' || url.origin !== self.location.origin ||
      !STATIC_ASSETS.has(url.pathname) || url.search || request.headers.has('Authorization')) return;

  event.respondWith(caches.open(CACHE_NAME).then(async (cache) => {
    const cached = await cache.match(request);
    if (cached) return cached;
    const response = await fetch(request);
    if (response.ok && response.type === 'basic' &&
        !/no-store|private/i.test(response.headers.get('Cache-Control') || '')) {
      await cache.put(request, response.clone());
    }
    return response;
  }));
});

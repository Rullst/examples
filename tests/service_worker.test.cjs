const { test } = require('node:test');
const vm = require('node:vm');
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');

for (const worker of ['../static/sw.js', '../blueprints/lms/static/sw.js']) test(`offline storage excludes sensitive pages and clears only obsolete showcase caches: ${worker}`, async () => {
    const listeners = {};
    let stored = 0;
    const deleted = [];
    const sandbox = {
        URL,
        self: {
            location: { origin: 'https://showcase.rullst.win' },
            addEventListener: (name, fn) => { listeners[name] = fn; },
            clients: { claim: async () => {} }
        },
        caches: {
            keys: async () => ['rullst-shell-v1', 'rullst-shell-v2', 'unrelated-cache'],
            delete: async name => { deleted.push(name); },
            open: async () => ({ match: async () => null, put: async () => { stored++; } })
        },
        fetch: async () => ({ ok: true, type: 'basic', headers: new Headers(), clone() { return this; } })
    };
    vm.runInNewContext(fs.readFileSync(path.join(__dirname, worker), 'utf8'), sandbox);

    const attempt = async (url, init = {}) => {
        let intercepted = false;
        let pending;
        listeners.fetch({
            request: new Request(new URL(url, sandbox.self.location.origin), init),
            respondWith(promise) { intercepted = true; pending = promise; }
        });
        if (pending) await pending;
        return intercepted;
    };

    for (const url of ['/', '/nexus', '/nexus/chat', '/studio/cache', '/api/showcase-chat', '/api/lms-chat', '/lessons/1/play', '/dashboard', '/privacy',
        'https://elsewhere.test/static/htmx.js', '/static/htmx.js?private=1']) {
        assert.equal(await attempt(url), false, url);
    }
    assert.equal(await attempt('/static/htmx.js', { headers: { Authorization: 'Basic fixture' } }), false);
    assert.equal(await attempt('/static/htmx.js', { method: 'POST' }), false);
    assert.equal(await attempt('/static/htmx.js'), true);
    assert.equal(stored, 1);

    sandbox.fetch = async () => ({ ok: true, type: 'basic', headers: new Headers({ 'Cache-Control': 'private, no-store' }) });
    await attempt('/static/htmx.js');
    assert.equal(stored, 1, 'Private responses must not be stored');

    let activation;
    listeners.activate({ waitUntil(promise) { activation = promise; } });
    await activation;
    assert.deepEqual(deleted, ['rullst-shell-v1']);
});

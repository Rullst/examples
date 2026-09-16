// Real Chromium check for the admin UI. Credentials arrive over stdin and are
// never placed in arguments, environment variables, browser URLs or output.
import { spawn } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const input = JSON.parse(await new Promise(resolve => {
  let data = '';
  process.stdin.setEncoding('utf8');
  process.stdin.on('data', chunk => { data += chunk; });
  process.stdin.on('end', () => resolve(data));
}));

const allowedOrigins = new Set([
  'https://showcase.rullst.win',
  'https://lms.rullst.win',
  'https://portfolio.rullst.win'
]);
let liveOrigin;
try {
  liveOrigin = new URL(input.origin);
} catch {
  console.error('Real-browser verification failed: invalid application origin.');
  process.exit(1);
}
if (!allowedOrigins.has(liveOrigin.origin) || liveOrigin.pathname !== '/') {
  console.error('Real-browser verification failed: application origin is not allowlisted.');
  process.exit(1);
}

const candidates = [
  process.env.CHROME_BIN,
  '/usr/bin/google-chrome',
  '/usr/bin/google-chrome-stable',
  '/usr/bin/chromium',
  'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
  'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe'
].filter(Boolean);
const chromePath = candidates.find(existsSync);
if (!chromePath) {
  console.error('Real-browser verification failed: Chromium is unavailable.');
  process.exit(1);
}

const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
let stage = 'starting Chromium';
let proxyServer;
let browserOrigin;
let profile;
let chrome;
let proxyRequestCount = 0;
let lastProxyStatus;
let lastProxyFailure;
let mobileLayoutFailures = [];
let mobileLayoutMetrics;

async function startLoopbackProxy() {
  const authorization = `Basic ${Buffer.from(`${input.username}:${input.password}`, 'utf8').toString('base64')}`;
  const excludedRequestHeaders = new Set([
    'accept-encoding', 'authorization', 'connection', 'content-length', 'host',
    'proxy-authorization', 'transfer-encoding', 'upgrade'
  ]);
  const excludedResponseHeaders = new Set([
    'connection', 'content-encoding', 'content-length', 'set-cookie',
    'transfer-encoding', 'upgrade'
  ]);
  const server = createServer(async (request, response) => {
    proxyRequestCount += 1;
    try {
      const target = new URL(request.url || '/', liveOrigin);
      if (target.origin !== liveOrigin.origin) throw new Error('cross-origin proxy target');
      const headers = new Headers();
      for (const [name, value] of Object.entries(request.headers)) {
        if (excludedRequestHeaders.has(name) || value === undefined) continue;
        headers.set(name, Array.isArray(value) ? value.join(', ') : value);
      }
      headers.set('accept-encoding', 'identity');
      headers.set('authorization', authorization);
      const chunks = [];
      for await (const chunk of request) chunks.push(chunk);
      const body = chunks.length ? Buffer.concat(chunks) : undefined;
      const upstream = await fetch(target, {
        method: request.method,
        headers,
        body: request.method === 'GET' || request.method === 'HEAD' ? undefined : body,
        redirect: 'manual'
      });
      lastProxyStatus = upstream.status;
      response.statusCode = upstream.status;
      upstream.headers.forEach((value, name) => {
        if (!excludedResponseHeaders.has(name)) response.setHeader(name, value);
      });
      const cookies = upstream.headers.getSetCookie?.() || [];
      if (cookies.length) {
        // The disposable proxy is HTTP loopback. Production still sets Secure;
        // strip it (and any live Domain) only from the browser-test copy.
        const loopbackCookies = cookies.map(cookie => cookie
          .replace(/;\s*Secure/gi, '')
          .replace(/;\s*Domain=[^;]+/gi, ''));
        response.setHeader('set-cookie', loopbackCookies);
      }
      response.end(Buffer.from(await upstream.arrayBuffer()));
    } catch {
      lastProxyFailure = 'upstream request failed';
      response.statusCode = 502;
      response.setHeader('content-type', 'text/plain; charset=utf-8');
      response.end('Live application proxy request failed.');
    }
  });
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', resolve);
  });
  proxyServer = server;
  const address = server.address();
  if (!address || typeof address === 'string') throw new Error('invalid proxy address');
  return `http://127.0.0.1:${address.port}`;
}

async function devtoolsPage() {
  for (let attempt = 0; attempt < 200; attempt += 1) {
    try {
      const [port] = readFileSync(join(profile, 'DevToolsActivePort'), 'utf8').trim().split(/\r?\n/);
      if (!/^\d+$/.test(port)) throw new Error('invalid DevTools port');
      const pages = await fetch(`http://127.0.0.1:${port}/json/list`).then(r => r.json());
      const appPage = pages.find(page => page.type === 'page' &&
        typeof page.url === 'string' && page.url.startsWith(browserOrigin));
      if (appPage?.webSocketDebuggerUrl) return appPage.webSocketDebuggerUrl;
    } catch {}
    if (chrome.exitCode !== null) break;
    await pause(100);
  }
  throw new Error('devtools unavailable');
}

async function run() {
  stage = 'starting the loopback proxy';
  browserOrigin = await startLoopbackProxy();
  stage = 'starting Chromium';
  profile = mkdtempSync(join(tmpdir(), 'rullst-browser-'));
  chrome = spawn(chromePath, [
    '--headless=new', '--no-sandbox', '--disable-gpu', '--disable-dev-shm-usage', '--no-first-run',
    // The browser talks to the public app only through its own allowlisted
    // loopback proxy. Do not let a runner system proxy intercept that hop.
    '--no-proxy-server',
    // DevTools is bound to loopback and the disposable profile below. Current
    // Chrome versions otherwise reject Node's WebSocket client by Origin.
    '--remote-allow-origins=*',
    '--remote-debugging-address=127.0.0.1', '--remote-debugging-port=0',
    `--user-data-dir=${profile}`, `${browserOrigin}/nexus/chat`
  ], { stdio: 'ignore' });
  stage = 'discovering the DevTools target';
  const debuggerUrl = await devtoolsPage();
  stage = 'opening the DevTools WebSocket';
  const socket = new WebSocket(debuggerUrl);
  await new Promise((resolve, reject) => {
    socket.addEventListener('open', resolve, { once: true });
    socket.addEventListener('error', reject, { once: true });
  });
  let id = 0;
  let lastDocumentStatus;
  let lastDocumentError;
  let expectedDocumentUrl;
  let currentDocumentRequestId;
  const pending = new Map();
  socket.addEventListener('message', event => {
    const message = JSON.parse(event.data);
    if (message.method === 'Network.requestWillBeSent' && message.params.type === 'Document' &&
        message.params.request.url === expectedDocumentUrl) {
      currentDocumentRequestId = message.params.requestId;
      return;
    }
    if (message.method === 'Network.responseReceived' && message.params.type === 'Document' &&
        message.params.requestId === currentDocumentRequestId) {
      lastDocumentStatus = message.params.response.status;
      return;
    }
    if (message.method === 'Network.loadingFailed' && message.params.type === 'Document' &&
        message.params.requestId === currentDocumentRequestId) {
      lastDocumentError = message.params.errorText;
      return;
    }
    if (!message.id || !pending.has(message.id)) return;
    const { resolve, reject } = pending.get(message.id);
    pending.delete(message.id);
    if (message.error) reject(new Error('devtools command failed'));
    else resolve(message.result);
  });
  const send = (method, params = {}) => new Promise((resolve, reject) => {
    const commandId = ++id;
    pending.set(commandId, { resolve, reject });
    socket.send(JSON.stringify({ id: commandId, method, params }));
  });
  const evaluate = async expression => {
    const result = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
    if (result.exceptionDetails) throw new Error('browser expression failed');
    return result.result.value;
  };
  const waitFor = async (expression, seconds = 45) => {
    for (let attempt = 0; attempt < seconds * 4; attempt += 1) {
      if (await evaluate(expression)) return;
      await pause(250);
    }
    throw new Error('browser condition timed out');
  };

  await send('Network.enable');
  await send('Page.enable');
  await send('Emulation.setDeviceMetricsOverride', {
    width: 390,
    height: 844,
    deviceScaleFactor: 3,
    mobile: true
  });

  for (const [panel, page] of [['nexus', 'chat'], ['studio', 'ai']]) {
    stage = `${panel} page load`;
    lastDocumentStatus = undefined;
    lastDocumentError = undefined;
    currentDocumentRequestId = undefined;
    expectedDocumentUrl = `${browserOrigin}/${panel}/${page}`;
    const currentDocumentUrl = await evaluate('location.href');
    if (currentDocumentUrl !== expectedDocumentUrl) {
      // Keep subsequent panel changes in the real document. Recent headless
      // Chrome builds can abort the first loopback Page.navigate before I/O.
      await evaluate(`location.assign(${JSON.stringify(expectedDocumentUrl)})`);
    }
    try {
      await waitFor("document.readyState === 'complete' && !!document.querySelector('#rullst-admin-ai form')", 20);
    } catch {
      const safeError = /^net::ERR_[A-Z0-9_]+$/.test(lastDocumentError || '')
        ? `, ${lastDocumentError}` : '';
      const status = Number.isInteger(lastDocumentStatus) ? lastDocumentStatus : 'unknown';
      stage = `${panel} page load (HTTP ${status}${safeError})`;
      throw new Error('admin page did not load');
    }
    stage = `${panel} UI contract`;
    const ui = await evaluate(`({
      english: !/(Voltar|Enviar|Sua pergunta|Olá|Portuguese)/.test(document.body.innerText),
      launcher: !!document.querySelector('a[style*="position:fixed"][aria-label*="AI"]'),
      token: document.querySelector('[name="_token"]')?.value?.length >= 16,
      mobile: (() => {
        const textarea = document.querySelector('#rullst-admin-ai textarea');
        const button = document.querySelector('#rullst-admin-ai form button');
        return innerWidth === 390 && document.documentElement.scrollWidth <= innerWidth + 1 &&
          textarea && parseFloat(getComputedStyle(textarea).fontSize) >= 16 &&
          button && button.getBoundingClientRect().height >= 44;
      })()
    })`);
    if (!ui.english || ui.launcher || !ui.token || !ui.mobile) throw new Error('admin UI contract failed');
    stage = `${panel} form submission`;
    await evaluate(`(() => {
      const form = document.querySelector('#rullst-admin-ai form');
      form.querySelector('textarea').value = 'In English only, explain this panel in one short sentence.';
      form.requestSubmit();
      return true;
    })()`);
    stage = `${panel} AI response`;
    await waitFor("document.querySelectorAll('.ai-message.ai-assistant .rullst-ai-prose').length > 0");
    stage = `${panel} denial check`;
    const denied = await evaluate("/(access denied|security token expired|authentication expired)/i.test(document.querySelector('.ai-messages').innerText)");
    if (denied) throw new Error('admin inference denied');
    console.log(`${input.app}: ${panel} real-browser inference verified.`);
  }

  const publicChat = {
    'rullst-showcase': {
      launcher: '#showcase-crab-launcher', drawer: '#showcase-ai-drawer',
      input: '#showcase-drawer-input', button: '#showcase-drawer-form button',
      credentials: '.sandbox-credential code'
    },
    'rullst-portfolio': {
      launcher: '#ai-crab-launcher', drawer: '#ai-drawer',
      input: '#ai-message-input', button: '#ai-chat-form button'
    },
    'rullst-lms': {
      launcher: '#lms-crab-launcher', drawer: '#lms-ai-drawer',
      input: '#lms-message-input', button: '#lms-chat-form button'
    }
  }[input.app];
  if (!publicChat) throw new Error('unknown public chat contract');

  stage = 'public mobile chat page load';
  lastDocumentStatus = undefined;
  lastDocumentError = undefined;
  currentDocumentRequestId = undefined;
  expectedDocumentUrl = `${browserOrigin}/`;
  await evaluate(`location.assign(${JSON.stringify(expectedDocumentUrl)})`);
  await waitFor(`document.readyState === 'complete' && !!document.querySelector(${JSON.stringify(publicChat.launcher)})`, 20);
  stage = 'public mobile chat layout';
  await evaluate(`document.querySelector(${JSON.stringify(publicChat.launcher)}).click()`);
  await waitFor(`getComputedStyle(document.querySelector(${JSON.stringify(publicChat.drawer)})).display !== 'none'`, 5);
  // Measure the settled layout, not the temporary scale/translation used by
  // the 250 ms opening animation in all three public chat drawers.
  await pause(350);
  const mobileResult = await evaluate(`(() => {
    const drawer = document.querySelector(${JSON.stringify(publicChat.drawer)});
    const field = document.querySelector(${JSON.stringify(publicChat.input)});
    const button = document.querySelector(${JSON.stringify(publicChat.button)});
    const rect = drawer?.getBoundingClientRect();
    const fieldRect = field?.getBoundingClientRect();
    const buttonRect = button?.getBoundingClientRect();
    const fieldFont = field ? parseFloat(getComputedStyle(field).fontSize) : 0;
    const bodyOverflow = getComputedStyle(document.body).overflow;
    return {
      checks: {
        fits: !!rect && rect.left >= -1 && rect.right <= innerWidth + 1 &&
          rect.top >= -1 && rect.bottom <= innerHeight + 1,
        noPageOverflow: document.documentElement.scrollWidth <= innerWidth + 1,
        touchInput: !!fieldRect && fieldFont >= 16 && fieldRect.height >= 44,
        touchButton: !!buttonRect && buttonRect.height >= 44,
        backgroundLocked: bodyOverflow === 'hidden',
        credentials: ${publicChat.credentials
          ? `(() => {
            const credentials = Array.from(document.querySelectorAll(${JSON.stringify(publicChat.credentials)}));
            return credentials.length === 2 && credentials.every(item => {
              const itemRect = item.getBoundingClientRect();
              const style = getComputedStyle(item);
              return itemRect.width > 0 && itemRect.height > 0 && style.visibility !== 'hidden';
            });
          })()`
          : 'true'}
      },
      metrics: {
        viewportWidth: innerWidth, viewportHeight: innerHeight,
        drawerLeft: Math.round(rect?.left || 0), drawerTop: Math.round(rect?.top || 0),
        drawerRight: Math.round(rect?.right || 0), drawerBottom: Math.round(rect?.bottom || 0),
        inputFont: fieldFont, inputHeight: Math.round(fieldRect?.height || 0),
        buttonHeight: Math.round(buttonRect?.height || 0), bodyOverflow
      }
    };
  })()`);
  mobileLayoutMetrics = mobileResult.metrics;
  mobileLayoutFailures = Object.entries(mobileResult.checks)
    .filter(([, passed]) => !passed)
    .map(([name]) => name);
  if (mobileLayoutFailures.length) throw new Error('public mobile chat contract failed');
  console.log(`${input.app}: public mobile chat layout verified.`);
  socket.close();
}

try {
  await run();
} catch {
  const proxyStatus = Number.isInteger(lastProxyStatus) ? lastProxyStatus : 'none';
  const proxyFailure = lastProxyFailure ? ', upstream failure' : '';
  console.error(`Browser proxy diagnostic: requests=${proxyRequestCount}, last status=${proxyStatus}${proxyFailure}.`);
  if (mobileLayoutFailures.length) {
    console.error(`Mobile layout diagnostic: ${mobileLayoutFailures.join(',')}.`);
    console.error(`Mobile metrics: ${JSON.stringify(mobileLayoutMetrics)}.`);
  }
  console.error(`Real-browser admin verification failed during ${stage}; no credentials or response bodies logged.`);
  process.exitCode = 1;
} finally {
  if (chrome) {
    chrome.kill('SIGKILL');
    await new Promise(resolve => {
      if (chrome.exitCode !== null) return resolve();
      chrome.once('exit', resolve);
      setTimeout(resolve, 2000);
    });
  }
  if (proxyServer) await new Promise(resolve => proxyServer.close(resolve));
  if (profile) rmSync(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
}

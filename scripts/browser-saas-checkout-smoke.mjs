// Real Chromium check for the SaaS session and hosted Stripe Checkout handoff.
// The account is synthetic, contains no personal data and uses Stripe Test Mode.
import { spawn } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { randomUUID } from 'node:crypto';

const allowedOrigins = new Set(['https://saas-staging.rullst.win']);
const origin = process.argv[2] || 'https://saas-staging.rullst.win';
if (!allowedOrigins.has(origin)) {
  console.error('SaaS browser verification failed: origin is not allowlisted.');
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
  console.error('SaaS browser verification failed: Chromium is unavailable.');
  process.exit(1);
}

const pause = milliseconds => new Promise(resolve => setTimeout(resolve, milliseconds));
let stage = 'starting Chromium';
let profile;
let chrome;
let socket;
let sawCspFormActionViolation = false;
let sawStripeNavigation = false;

async function devtoolsPage() {
  for (let attempt = 0; attempt < 200; attempt += 1) {
    try {
      const [port] = readFileSync(join(profile, 'DevToolsActivePort'), 'utf8').trim().split(/\r?\n/);
      if (!/^\d+$/.test(port)) throw new Error('invalid DevTools port');
      const pages = await fetch(`http://127.0.0.1:${port}/json/list`).then(response => response.json());
      const page = pages.find(candidate => candidate.type === 'page');
      if (page?.webSocketDebuggerUrl) return page.webSocketDebuggerUrl;
    } catch {}
    if (chrome.exitCode !== null) break;
    await pause(100);
  }
  throw new Error('DevTools unavailable');
}

async function run() {
  profile = mkdtempSync(join(tmpdir(), 'rullst-saas-browser-'));
  chrome = spawn(chromePath, [
    '--headless=new', '--no-sandbox', '--disable-gpu', '--disable-dev-shm-usage', '--no-first-run',
    '--no-proxy-server', '--remote-allow-origins=*',
    '--remote-debugging-address=127.0.0.1', '--remote-debugging-port=0',
    `--user-data-dir=${profile}`, `${origin}/register`
  ], { stdio: 'ignore' });

  stage = 'opening DevTools';
  socket = new WebSocket(await devtoolsPage());
  await new Promise((resolve, reject) => {
    socket.addEventListener('open', resolve, { once: true });
    socket.addEventListener('error', reject, { once: true });
  });

  let commandId = 0;
  const pending = new Map();
  socket.addEventListener('message', event => {
    const message = JSON.parse(event.data);
    if (message.method === 'Network.requestWillBeSent' &&
        message.params.type === 'Document') {
      try {
        if (new URL(message.params.request.url).hostname === 'checkout.stripe.com') {
          sawStripeNavigation = true;
        }
      } catch {}
    }
    if (message.method === 'Runtime.consoleAPICalled') {
      const text = message.params.args
        .map(argument => typeof argument.value === 'string' ? argument.value : '')
        .join(' ');
      if (/content security policy/i.test(text) && /form-action/i.test(text)) {
        sawCspFormActionViolation = true;
      }
    }
    if (message.method === 'Log.entryAdded') {
      const text = message.params.entry?.text || '';
      if (/content security policy/i.test(text) && /form-action/i.test(text)) {
        sawCspFormActionViolation = true;
      }
    }
    if (!message.id || !pending.has(message.id)) return;
    const { resolve, reject } = pending.get(message.id);
    pending.delete(message.id);
    if (message.error) reject(new Error('DevTools command failed'));
    else resolve(message.result);
  });

  const send = (method, params = {}) => new Promise((resolve, reject) => {
    const id = ++commandId;
    pending.set(id, { resolve, reject });
    socket.send(JSON.stringify({ id, method, params }));
  });
  const evaluate = async expression => {
    const result = await send('Runtime.evaluate', {
      expression,
      returnByValue: true,
      awaitPromise: true
    });
    if (result.exceptionDetails) throw new Error('browser expression failed');
    return result.result.value;
  };
  const waitFor = async (expression, seconds = 30) => {
    for (let attempt = 0; attempt < seconds * 4; attempt += 1) {
      try {
        if (await evaluate(expression)) return;
      } catch {}
      await pause(250);
    }
    throw new Error('browser condition timed out');
  };

  await send('Network.enable');
  await send('Page.enable');
  await send('Runtime.enable');
  await send('Log.enable');

  stage = 'loading registration';
  await waitFor("document.readyState === 'complete' && !!document.querySelector('form[action=\"/register\"]')", 45);
  const suffix = randomUUID().replaceAll('-', '');
  const email = `checkout-smoke-${suffix}@example.invalid`;
  const password = `Checkout-${randomUUID()}-A9!`;
  stage = 'registering synthetic account';
  await evaluate(`(() => {
    const form = document.querySelector('form[action="/register"]');
    form.elements.name.value = 'Checkout Smoke Test';
    form.elements.email.value = ${JSON.stringify(email)};
    form.elements.password.value = ${JSON.stringify(password)};
    form.elements.certificate_name_acknowledgement.checked = true;
    form.requestSubmit();
    return true;
  })()`);
  await waitFor("location.pathname === '/dashboard' && document.body.innerText.includes('Welcome,') && !!document.querySelector('form[action=\"/billing/checkout\"]')", 45);

  stage = 'checking direct dashboard checkout';
  const accountContext = await evaluate(`({
    signedIn: document.body.innerText.includes('Welcome,'),
    directLabel: document.body.innerText.includes('Open Stripe test checkout'),
    pricingDetour: !!document.querySelector('a[href="/pricing"]'),
    csrf: document.querySelector('form[action="/billing/checkout"] [name="_token"]')?.value?.length >= 16
  })`);

  stage = 'submitting direct dashboard checkout';
  await evaluate(`(() => {
    const form = document.querySelector('form[action="/billing/checkout"]');
    form.elements.purchase_authority.checked = true;
    form.requestSubmit();
    return true;
  })()`);

  for (let attempt = 0; attempt < 180 && !sawStripeNavigation; attempt += 1) {
    await pause(250);
  }
  if (!sawStripeNavigation) {
    if (sawCspFormActionViolation) stage = 'Stripe handoff blocked by CSP form-action';
    throw new Error('Stripe navigation was not observed');
  }
  if (!accountContext.signedIn || !accountContext.directLabel || accountContext.pricingDetour || !accountContext.csrf) {
    stage = 'authenticated dashboard checkout state';
    throw new Error('authenticated dashboard checkout state is inconsistent');
  }

  console.log('SaaS staging: direct authenticated dashboard handoff to Stripe verified in Chromium.');
}

try {
  await run();
} catch {
  console.error(`SaaS browser verification failed during ${stage}; no credentials or Checkout URL logged.`);
  process.exitCode = 1;
} finally {
  if (socket) socket.close();
  if (chrome) {
    chrome.kill('SIGKILL');
    await new Promise(resolve => {
      if (chrome.exitCode !== null) return resolve();
      chrome.once('exit', resolve);
      setTimeout(resolve, 2000);
    });
  }
  if (profile) {
    try {
      rmSync(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
    } catch {}
  }
}

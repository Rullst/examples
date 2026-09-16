// Real Chromium check for the admin UI. Credentials arrive over stdin and are
// never placed in arguments, environment variables, browser URLs or output.
import { spawn } from 'node:child_process';
import { existsSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const input = JSON.parse(await new Promise(resolve => {
  let data = '';
  process.stdin.setEncoding('utf8');
  process.stdin.on('data', chunk => { data += chunk; });
  process.stdin.on('end', () => resolve(data));
}));

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

const profile = mkdtempSync(join(tmpdir(), 'rullst-browser-'));
const port = 19000 + (process.pid % 1000);
const chrome = spawn(chromePath, [
  '--headless=new', '--no-sandbox', '--disable-gpu', '--no-first-run',
  `--remote-debugging-port=${port}`, `--user-data-dir=${profile}`, 'about:blank'
], { stdio: 'ignore' });

const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
let stage = 'starting Chromium';

async function devtoolsPage() {
  for (let attempt = 0; attempt < 50; attempt += 1) {
    try {
      const pages = await fetch(`http://127.0.0.1:${port}/json/list`).then(r => r.json());
      if (pages[0]?.webSocketDebuggerUrl) return pages[0].webSocketDebuggerUrl;
    } catch {}
    await pause(100);
  }
  throw new Error('devtools unavailable');
}

async function run() {
  stage = 'opening the DevTools connection';
  const socket = new WebSocket(await devtoolsPage());
  await new Promise((resolve, reject) => {
    socket.addEventListener('open', resolve, { once: true });
    socket.addEventListener('error', reject, { once: true });
  });
  let id = 0;
  const pending = new Map();
  socket.addEventListener('message', event => {
    const message = JSON.parse(event.data);
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
  const basic = Buffer.from(`${input.username}:${input.password}`, 'utf8').toString('base64');
  await send('Network.setExtraHTTPHeaders', { headers: { Authorization: `Basic ${basic}` } });

  for (const [panel, page] of [['nexus', 'chat'], ['studio', 'ai']]) {
    stage = `${panel} page load`;
    await send('Page.navigate', { url: `${input.origin}/${panel}/${page}` });
    await waitFor("document.readyState === 'complete' && !!document.querySelector('#rullst-admin-ai form')", 20);
    stage = `${panel} UI contract`;
    const ui = await evaluate(`({
      english: !/(Voltar|Enviar|Sua pergunta|Olá|Portuguese)/.test(document.body.innerText),
      launcher: !!document.querySelector('a[style*="position:fixed"][aria-label*="AI"]'),
      token: document.querySelector('[name="_token"]')?.value?.length >= 16
    })`);
    if (!ui.english || ui.launcher || !ui.token) throw new Error('admin UI contract failed');
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
  socket.close();
}

try {
  await run();
} catch {
  console.error(`Real-browser admin verification failed during ${stage}; no credentials or response bodies logged.`);
  process.exitCode = 1;
} finally {
  chrome.kill('SIGKILL');
  rmSync(profile, { recursive: true, force: true });
}

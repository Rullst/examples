"""Read-only Azure checks and bounded live AI smoke tests; never log secrets/bodies."""

import argparse
import base64
import http.cookiejar
import json
import re
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request


APPS = {
    "rullst-showcase": ("https://showcase.rullst.win", "/api/showcase-chat", "rullst_admin"),
    "rullst-lms": ("https://lms.rullst.win", "/api/lms-chat", "admin"),
    "rullst-portfolio": ("https://portfolio.rullst.win", "/api/chat", "admin"),
}


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def azure(app, *args):
    result = subprocess.run(
        ["az", "containerapp", *args, "--name", app, "--resource-group", "rullst-rg",
         "--only-show-errors", "--output", "json"],
        capture_output=True, text=True, timeout=90,
    )
    require(result.returncode == 0, "Azure read failed; check deployment identity permissions.")
    return json.loads(result.stdout)


def configuration(app, properties):
    containers = properties["template"]["containers"]
    require(len(containers) == 1, "Expected exactly one application container.")
    entries = {item["name"]: item for item in containers[0].get("env", [])}

    def value(name):
        entry = entries.get(name, {})
        if entry.get("secretRef"):
            ref = entry["secretRef"]
            require(re.fullmatch(r"[a-zA-Z0-9-]+", ref), "Invalid secret reference.")
            # Only the referenced value is returned; it stays in process memory.
            return azure(app, "secret", "list", "--show-values", "--query",
                         f"[?name=='{ref}'].value | [0]") or ""
        return entry.get("value", "")

    password = value("NEXUS_ADMIN_PASSWORD")
    require(len(password) >= 16, "Set NEXUS_ADMIN_PASSWORD (at least 16 characters) before deployment.")
    username = value("NEXUS_ADMIN_USERNAME") or APPS[app][2]
    key = next((value(name).strip().strip("\"'") for name in
                ("GROQ_API_KEY", "GROQ_KEY", "GROQ_APIKEY", "GROQ_TOKEN") if name in entries), "")
    require(key and not key.startswith("mock_"), "A real server-side Groq credential is required.")
    return username, password


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        # Never forward administrator credentials to a redirect destination.
        return None


def rendered_online(body):
    require('class="rullst-ai-prose"' in body, "Response is missing the new sanitized renderer.")
    require(re.search(r"<(p|ul|ol|table|h[1-6])(?:>| )", body), "Response has no formatted content.")
    require(not any(text in body.lower() for text in (
        "offline assistant", "assistente offline", "temporarily unavailable",
        "generative ai is currently unavailable", "please retry in a minute",
        "please rephrase your request",
    )), "AI returned an offline, blocked, busy or unavailable response.")


def smoke(app, username, password):
    origin, public_path, _ = APPS[app]
    cookies = http.cookiejar.CookieJar()
    client = urllib.request.build_opener(NoRedirect(), urllib.request.HTTPCookieProcessor(cookies))
    authorization = "Basic " + base64.b64encode(f"{username}:{password}".encode()).decode()

    def request(path, message=None, authenticated=False, cross_origin=False):
        headers = {"Cache-Control": "no-cache"}
        if authenticated:
            headers["Authorization"] = authorization
        data = None
        if message is not None:
            headers.update({"Origin": "https://invalid.example" if cross_origin else origin,
                            "X-Rullst-AI": "1", "Sec-Fetch-Site": "same-origin",
                            "Content-Type": "application/x-www-form-urlencoded"})
            token = next((c.value for c in cookies if c.name == "rullst_csrf"), "")
            headers["X-CSRF-Token"] = token
            data = urllib.parse.urlencode({"message": message, "_token": token}).encode()
        req = urllib.request.Request(origin + path, data=data, headers=headers)
        try:
            response = client.open(req, timeout=40)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            return response.code, response.read(512 * 1024).decode("utf-8", errors="replace")

    require(request("/")[0] == 200, "Public homepage is not healthy.")
    prompt = "Explain this application's purpose briefly, with two bullet points and bold labels."
    status, body = request(public_path, prompt)
    require(status == 200, f"Public chat returned HTTP {status}.")
    rendered_online(body)
    print(f"{app}: public chat returns formatted AI content.", flush=True)
    for panel, page in (("nexus", "chat"), ("studio", "ai")):
        path = f"/{panel}/{page}"
        require(request(path)[0] == 401, f"{panel}: anonymous access must be denied.")
        status, body = request(path, authenticated=True)
        require(status == 200 and 'id="rullst-admin-ai"' in body,
                f"{panel}: authenticated AI page is not deployed (HTTP {status}).")
        query = f"/{panel}/copilot/query"
        require(request(query, prompt, True, True)[0] == 403,
                f"{panel}: cross-origin inference must be rejected.")
        status, body = request(query, prompt, True)
        require(status == 200, f"{panel}: inference returned HTTP {status}.")
        rendered_online(body)
        print(f"{app}: {panel} authentication, CSRF and formatted AI reply verified.", flush=True)

    browser = subprocess.run(
        ["node", "scripts/browser-admin-smoke.mjs"],
        input=json.dumps({"app": app, "origin": origin, "username": username, "password": password}),
        capture_output=True, text=True, timeout=120,
    )
    require(browser.returncode == 0, "Real-browser Nexus/Studio verification failed; no credentials logged.")
    for line in browser.stdout.splitlines():
        require(line.startswith(f"{app}: "), "Unexpected browser verification output.")
        print(line, flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("app", choices=APPS)
    parser.add_argument("--preflight", action="store_true")
    parser.add_argument("--image")
    args = parser.parse_args()
    properties = azure(args.app, "show")["properties"]
    username, password = configuration(args.app, properties)
    if args.preflight:
        print(f"{args.app}: required administrator and provider configuration present.")
        return
    require(args.image, "An immutable expected image is required.")
    for attempt in range(24):
        properties = azure(args.app, "show")["properties"]
        if (properties.get("latestRevisionName")
                and properties["latestRevisionName"] == properties.get("latestReadyRevisionName")
                and properties["template"]["containers"][0]["image"] == args.image):
            break
        require(attempt < 23, "Azure has not made the expected image ready.")
        print(f"{args.app}: waiting for the expected Azure revision...", flush=True)
        time.sleep(10)
    smoke(args.app, username, password)


if __name__ == "__main__":
    try:
        main()
    except RuntimeError as error:
        print(f"::error::{error}", file=sys.stderr)
        sys.exit(1)
    except Exception:
        # HTTP/CLI exceptions can contain sensitive response/configuration data.
        print("::error::Deployment verification failed; no response bodies or credentials logged.", file=sys.stderr)
        sys.exit(1)

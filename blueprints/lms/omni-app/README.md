# Rullst LMS — Rullst Omni web shell

This directory packages the canonical Rullst web application at `https://rullst-lms.redpond-24d9228d.eastus.azurecontainerapps.io`
for desktop, Android and iOS through Tauri. The local bootstrap contains no
application data and exposes no Tauri IPC API to the remote page. Native-side
navigation accepts only the packaged bootstrap and the exact backend origin.

The scaffold uses the application-owned identifier supplied to `make:omni`.

## Run locally

```bash
cargo rullst omni desktop
cargo rullst omni android
cargo rullst omni ios
```

Android requires its SDK/NDK; iOS requires macOS and Xcode. Distributable apps
must use an HTTPS endpoint reachable from the real device. The web backend
remains responsible for authentication, authorization, CSP, CSRF and data.

## Intentional security boundary

- Cross-origin navigation is blocked. OAuth and external links must use a
  reviewed system-browser/deep-link integration instead of weakening the
  navigation allowlist.
- Remote web content receives no privileged Tauri commands by default.
- Offline synchronization, push, biometrics and secure device storage are not
  implied by this web-shell profile; add and test only the capabilities used.

## Distribution checklist

1. confirm the application-owned identifier, version, icons and product metadata;
2. configure platform signing, provisioning and privacy/usage declarations;
3. test the production HTTPS backend and authentication on physical devices;
4. complete accessibility, offline/error and data-retention testing;
5. use the platform beta channel before production review and publication.

The generated shell and CI compilation are packaging evidence, not proof of
App Store or Play acceptance. Signing credentials, privacy answers, native
capability policy, store publication and review remain application-owned.

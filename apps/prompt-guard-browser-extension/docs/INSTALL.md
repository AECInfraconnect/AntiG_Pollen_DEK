# Install Pollek Prompt Guard Browser Connector

The connector supports Chromium-family browsers and Safari Web Extensions through separate packages generated from the same source.

## Chrome and Chromium Browsers

Build:

```bash
npm --prefix apps/prompt-guard-browser-extension run build:chromium
```

Load for local testing:

1. Open `chrome://extensions`.
2. Enable Developer mode.
3. Choose **Load unpacked**.
4. Select `apps/prompt-guard-browser-extension/dist/chromium`.

For Chrome Web Store submission, upload `apps/prompt-guard-browser-extension/packages/pollek-prompt-guard-chromium.zip`.

## Microsoft Edge

Build:

```bash
npm --prefix apps/prompt-guard-browser-extension run build:edge
```

Load for local testing:

1. Open `edge://extensions`.
2. Enable Developer mode.
3. Choose **Load unpacked**.
4. Select `apps/prompt-guard-browser-extension/dist/edge`.

For Microsoft Edge Add-ons submission, upload `apps/prompt-guard-browser-extension/packages/pollek-prompt-guard-edge.zip`.

## Safari on macOS

Build the Safari-compatible WebExtension source:

```bash
npm --prefix apps/prompt-guard-browser-extension run build:safari
```

Then convert/package with Xcode:

```bash
xcrun safari-web-extension-converter apps/prompt-guard-browser-extension/dist/safari-webextension --project-location apps/prompt-guard-browser-extension/safari-app --app-name "Pollek Prompt Guard Connector" --bundle-identifier "ai.pollek.promptguard"
```

Open the generated Xcode project, sign it with an Apple Developer account if needed, run the app, then enable the extension in Safari Settings > Extensions.

## About automatic installation

Chrome, Edge, and Safari do not allow a local web dashboard to silently install a browser extension. Users must approve installation, or an organization must deploy it with managed browser policy.

Pollek can prepare the package, open the right setup instructions, and detect connector telemetry after installation. Silent install is intentionally not part of this connector.

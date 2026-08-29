// Minimal stealth shim for agent-browser (Chrome for Testing).
// Removes the most common automation fingerprints so bot-walls treat
// this browser like a regular one.
Object.defineProperty(navigator, "webdriver", { get: () => undefined });

// Chrome for Testing ships without plugins; pretend to have some.
Object.defineProperty(navigator, "plugins", {
  get: () => [
    { name: "Chrome PDF Plugin" },
    { name: "Chrome PDF Viewer" },
    { name: "Native Client" },
  ],
});

Object.defineProperty(navigator, "languages", {
  get: () => ["en-US", "en"],
});

// Headless/自动化 Chrome 的 permissions.query 会暴露 Notification 异常
if (window.Notification && navigator.permissions) {
  const originalQuery = navigator.permissions.query.bind(navigator.permissions);
  navigator.permissions.query = (params) =>
    params && params.name === "notifications"
      ? Promise.resolve({ state: Notification.permission })
      : originalQuery(params);
}

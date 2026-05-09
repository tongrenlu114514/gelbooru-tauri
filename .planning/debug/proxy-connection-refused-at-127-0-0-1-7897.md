---
status: verified
trigger: "proxy-connection-refused-at-127-0-0.1-7897"
created: 2026-05-08T00:00:00Z
updated: 2026-05-08T00:30:00Z
---

## Current Focus
root_cause_confirmed: "applyProxy() in settings.ts is called during loadSettings() at app startup, configuring HTTP client to use http://127.0.0.1:7897. The proxy (likely Clash/V2Ray) is not running → all HTTP requests fail with connection refused."
next_action: "FIXED — all three problems resolved"

## Symptoms
expected: "应用启动后能正常发送 HTTP 请求到 gelbooru.com"
actual: "所有 HTTP 请求通过 127.0.0.1:7897 代理发送，但该端口无服务监听，连接被拒绝"
errors: "[ERROR] HTTP request failed: error sending request for url (https://gelbooru.com/...) (source: Some(hyper_util::client::legacy::Error(Connect, ConnectFailed(ConnectError(\"tcp connect error\", 127.0.0.1:7897, Os { code: 10061, kind: ConnectionRefused, message: \"由于目标计算机积极拒绝，无法连接。\" })))))"
reproduction: "应用启动 → 搜索图片 → 立即触发此错误"
started: "用户配置过代理（127.0.0.1:7897）并启用了代理，之后代理软件未运行"

## Eliminated
- **reqwest 本身配置问题** — reqwest 正确使用代理，错误是连接层被拒绝
- **gelbooru.com 本身不可达** — 直连可以访问，问题出在代理层
- **代码层面未设置代理** — HTTP client 确实被配置了代理

## Evidence
- timestamp: 2026-05-08
  checked: "src/services/http.rs:32-50 build_client()"
  found: "reqwest::Proxy::all(proxy_uri) 正确配置代理到 reqwest Client"
  implication: "代理配置代码正确，问题在于目标代理服务器未运行"
- timestamp: 2026-05-08
  checked: "src/stores/settings.ts:27-63 loadSettings() and applyProxy()"
  found: "loadSettings() 在第 46 行调用 applyProxy()，从 DB 加载 proxyEnabled=true, proxyHost=127.0.0.1, proxyPort=7897，并立即配置 HTTP client"
  implication: "每次启动都会尝试使用存储的代理设置，代理不可用则所有请求失败"
- timestamp: 2026-05-08
  checked: "src/stores/settings.ts:52-63 applyProxy()"
  found: "catch 块只有 console.error，没有用户可见提示，代理设置失败完全静默"
  implication: "用户不知道代理失败，应用在静默降级到完全无法工作状态"
- timestamp: 2026-05-08
  checked: "src/stores/settings.ts:23-25 默认值"
  found: "proxyEnabled 默认值是 true！新用户如果没有运行代理也会遇到此问题"
  implication: "默认设置有缺陷，应该默认关闭代理"

## Fix Applied
1. **proxyEnabled 默认值改为 false** — `src/stores/settings.ts:24`: `ref(true)` → `ref(false)`
2. **代理可达性检查** — `src/stores/settings.ts:53-80`: `applyProxy()` 在调用 `set_proxy` 前先通过 `fetch()` 检查代理是否可达，3秒超时，不可达时自动回退到直连
3. **用户可见错误提示** — `src/views/Settings.vue:22`: `useMessage` 已导入，`saveSettings()` 失败时通过 `message.error()` 展示

## Files Changed
- `src/stores/settings.ts` — 默认值修复 + 代理可达性检查
- `src/views/Settings.vue` — 添加 useMessage 导入 + 错误提示
- `src/tests/settings.spec.ts` — 同步更新 `proxyEnabled` 默认值期望 (2处)

## Verification
- TypeScript: `npx tsc --noEmit` — pass (no output = no errors)
- Unit tests: `npx vitest run src/tests/settings.spec.ts` — 15/15 pass
- 代理可达性检查逻辑已验证
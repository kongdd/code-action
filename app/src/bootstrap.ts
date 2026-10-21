// 入口：开发模式带 ?preview 参数时先注入模拟 IPC，便于在浏览器中检查排版。
export {};

if (import.meta.env.DEV && new URLSearchParams(location.search).has("preview")) {
  await import("./preview-mock");
}
await import("./main");

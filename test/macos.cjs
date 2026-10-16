const assert = require("node:assert/strict");
const { execFileSync, execSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const Module = require("node:module");
const { transformSync } = require("esbuild");

assert.equal(process.platform, "darwin", "此测试仅用于 macOS");

// 仅模拟 uTools 接口；数据库、文件系统和登录 shell 使用本机环境。
let settings;
let plugins;
class ListItem {
  constructor(title, description, data, icon) {
    Object.assign(this, { title, description, data, icon });
  }
}
const helper = {
  ListItem,
  Setting: {
    Get: name => settings[name],
    Init: (_, config) => {
      settings = Object.fromEntries(config.map(item => [item.name, item.default]));
      return {};
    },
  },
  InitPlugins: items => { plugins = items; },
};
const load = Module._load;
Module._load = function (id, ...args) {
  return id === "utools-helper" ? helper : load.call(this, id, ...args);
};
require.extensions[".ts"] = (module, filename) => {
  const { code } = transformSync(fs.readFileSync(filename, "utf8"), {
    loader: "ts", format: "cjs", target: "node16",
  });
  module._compile(code, filename);
};
global.utools = {
  getPath: () => path.join(os.homedir(), "Library", "Application Support"),
  dbStorage: { getItem: () => [] },
};
global.document = { addEventListener() {} };
global.alert = message => { throw new Error(message); };

async function main() {
  require("../src/preload.ts");
  assert.equal(plugins.length, 3);
  assert.equal(settings.shell, "zsh -l -c");
  assert.equal(settings.explorer, "open");
  assert.equal(settings.terminal, "");
  assert.ok(fs.existsSync(settings.db));
  console.log("PASS macOS 默认配置与插件初始化");

  const { GetFiles } = require("../src/files.ts");
  const history = await GetFiles(settings.db);
  assert.ok(history.length > 0);
  const items = await plugins[0].enter();
  assert.ok(items.length > 0);
  assert.ok(items.every(item => /^vscode-remote:/i.test(item.description)
    || fs.existsSync(item.description)));
  const sample = items.find(item => !/^vscode-remote:/i.test(item.description));
  assert.ok(sample);
  const query = sample.description.split("/").filter(Boolean).slice(-2).join(" ");
  const results = await plugins[0].search(query);
  assert.ok(results.some(item => item.description === sample.description));
  assert.ok(results.every(item => query.toLowerCase().split(/\s+/)
    .every(word => item.description.toLowerCase().includes(word))));
  console.log(`PASS 真实 VSCode 历史：${history.length} 条，可用 ${items.length} 条；多关键词搜索`);

  const { check_path, quoteArg, editorCmd, darwinTerminalScript } = require("../src/plugin.ts");
  assert.equal(check_path("file:///Users/test/My%20Project"), "/Users/test/My Project");
  assert.equal(check_path("file://localhost/Users/test/project"), "/Users/test/project");
  const target = '/tmp/中文 project\'s "$HOME"';
  assert.equal(execSync(`printf %s ${quoteArg(target)}`, { encoding: "utf8" }), target);
  const version = execSync(editorCmd(settings.code, "--version", settings.shell), {
    encoding: "utf8", timeout: 10000,
  });
  assert.match(version, /\d+\.\d+\.\d+/);
  console.log(`PASS 路径与 shell 转义；code ${version.split("\n")[0]}`);

  // 编译 AppleScript 验证语法，不打开窗口或触发自动化权限提示。
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "utools-macos-"));
  try {
    execFileSync("osacompile", ["-o", path.join(temp, "terminal.scpt"),
      "-e", darwinTerminalScript(target), "-e", 'tell application "Terminal" to activate']);
    assert.ok(fs.existsSync("/usr/bin/open"));
    console.log("PASS Terminal AppleScript 编译；Finder 命令存在（未测试界面操作）");
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
}
main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});

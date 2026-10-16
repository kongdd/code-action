const assert = require("node:assert/strict");
const { execFileSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const Module = require("node:module");
const { transformSync } = require("esbuild");

let settings;
let plugins;
let opened;
let hidden = false;
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
  getPath: () => os.tmpdir(),
  shellOpenPath: value => { opened = value; },
  hideMainWindow: () => { hidden = true; },
};
global.document = { addEventListener() {} };
global.alert = message => { throw new Error(message); };

async function main() {
  require("../src/preload.ts");
  assert.equal(plugins.length, 4);
  const plugin = plugins.find(item => item.code === "fs_action");
  assert.ok(plugin);
  const manifest = JSON.parse(fs.readFileSync(path.join(__dirname, "../plugin.json"), "utf8"));
  assert.ok(manifest.features.some(item => item.code === plugin.code));

  // 使用临时索引与真实 fs，不修改用户配置，也不实际打开应用。
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "utools-fs-"));
  const previousConfig = process.env.FS_CONFIG;
  try {
    const root = path.join(temp, "papers");
    fs.mkdirSync(root);
    const names = ["soil rain.pdf", "soil\nrain.txt", "Kong 2024.pdf", "-notes.txt"];
    names.forEach(name => fs.writeFileSync(path.join(root, name), ""));
    process.env.FS_CONFIG = path.join(temp, "config.toml");
    const tomlPath = value => JSON.stringify(value.replace(/\\/g, "/"));
    fs.writeFileSync(process.env.FS_CONFIG,
      `engine = "rust"\n[[index]]\nname = "test"\nroot = ${tomlPath(root)}\ndatabase = ${tomlPath(path.join(temp, "files.db"))}\n`);
    const cargoBinary = path.join(os.homedir(), ".cargo", "bin", process.platform === "win32" ? "fs.exe" : "fs");
    execFileSync(fs.existsSync(cargoBinary) ? cargoBinary : "fs",
      ["updatedb", "--no-progress"], { stdio: "pipe" });

    assert.deepEqual(await plugin.enter(), []);
    assert.deepEqual(await plugin.search("   "), []);
    const items = await plugin.search("soil rain");
    assert.equal(items.length, 2);
    assert.ok(items.some(item => item.data.includes("\n")));
    assert.ok(items.every(item => item.description === item.data && item.icon));
    assert.equal((await plugin.search("*.pdf")).length, 2);
    assert.equal((await plugin.search('"Kong 2024" exts:pdf'))[0].title, "Kong 2024.pdf");
    assert.equal((await plugin.search("path:papers -notes"))[0].title, "-notes.txt");
    assert.equal((await plugin.search("-notes"))[0].title, "-notes.txt");
    assert.deepEqual(await plugin.search("no-such-file"), []);
    plugin.select(items[0]);
    assert.equal(opened, items[0].data);
    assert.ok(hidden);

    fs.writeFileSync(process.env.FS_CONFIG, "invalid = [");
    const logError = console.error;
    console.error = () => {};
    let errors;
    try {
      errors = await plugin.search("soil");
    } finally {
      console.error = logError;
    }
    assert.equal(errors[0].title, "fs 搜索失败");
    plugin.select(errors[0]);
    assert.equal(opened, items[0].data);
    console.log("PASS fs 注册、真实索引搜索、表达式、特殊路径、打开及错误处理");
  } finally {
    if (previousConfig === undefined) delete process.env.FS_CONFIG;
    else process.env.FS_CONFIG = previousConfig;
    fs.rmSync(temp, { recursive: true, force: true });
  }
}
main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});

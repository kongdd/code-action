const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const output = path.join(__dirname, "../release");
const manifest = require(path.join(output, "plugin.json"));
assert.deepEqual(fs.readdirSync(output).sort(), ["dist", "icon", "layui", "plugin.json", "third_party"]);
assert.deepEqual(manifest, require("../plugin.json"));
for (const file of [
  manifest.preload, manifest.logo,
  ...manifest.features.map(feature => feature.icon).filter(Boolean),
  "third_party/sqljs/sql-wasm.js", "third_party/sqljs/sql-wasm.wasm",
  "layui/layui.all.js", "layui/layui.css",
]) {
  assert.ok(fs.existsSync(path.join(output, file)), `缺少资源：${file}`);
}

// 验证打包后的相对路径和入口，避免只在仓库中能够运行。
global.window = {};
global.document = { addEventListener() {} };
global.utools = { getPath: () => os.tmpdir() };
global.alert = message => { throw new Error(message); };
require(path.join(output, manifest.preload));
assert.deepEqual(Object.keys(window.exports).sort(), manifest.features.map(feature => feature.code).sort());

async function main() {
  const SQL = await require(path.join(output, "third_party/sqljs/sql-wasm"))();
  const db = new SQL.Database();
  assert.equal(db.exec("select 1")[0].values[0][0], 1);
  db.close();
  console.log("PASS 发布清单、资源路径、插件注册及 SQLite WASM 加载");
}
main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});

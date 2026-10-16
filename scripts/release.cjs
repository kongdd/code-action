const { cpSync, mkdirSync, rmSync } = require("node:fs");
const { join } = require("node:path");

const root = join(__dirname, "..");
const output = join(root, "release");
// 保持与仓库相同的运行时路径，只复制发布必需文件。
const files = [
  ["plugin.json", "plugin.json"],
  ["dist/preload.js", "dist/preload.js"],
  ["icon", "icon"],
  ["third_party/sqljs", "third_party/sqljs"],
  ["node_modules/utools-helper/dist/layui", "layui"],
];

rmSync(output, { recursive: true, force: true });
mkdirSync(output, { recursive: true });
for (const [source, target] of files) {
  cpSync(join(root, source), join(output, target), { recursive: true });
}
console.log("发布目录：release/；在 uTools 中导入 release/plugin.json 后打包。");

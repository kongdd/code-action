const assert = require("node:assert/strict");
const fs = require("node:fs");
const Module = require("node:module");
const { transformSync } = require("esbuild");

const storage = new Map();
let input;
let query;
class ListItem {
  constructor(title, description, data, icon) {
    Object.assign(this, { title, description, data, icon });
  }
}
const load = Module._load;
Module._load = function (id, ...args) {
  if (id === "utools-helper") return { ListItem, Setting: { Get: () => "" } };
  if (id === "child_process") return {
    ...load.call(this, id, ...args),
    execFile: Object.assign(() => {}, {
      [require("node:util").promisify.custom]: async (bin, args) => {
        query = args.at(-1);
        return { stdout: JSON.stringify([{ path: `/tmp/${query}.pdf` }]) };
      },
    }),
  };
  return load.call(this, id, ...args);
};
require.extensions[".ts"] = (module, filename) => {
  const { code } = transformSync(fs.readFileSync(filename, "utf8"), {
    loader: "ts", format: "cjs", target: "node16",
  });
  module._compile(code, filename);
};
global.document = { addEventListener() {} };
global.utools = {
  dbStorage: {
    getItem: key => storage.get(key),
    setItem: (key, value) => storage.set(key, value),
  },
  setSubInputValue: value => { input = value; },
  showNotification() {},
};
const { VSCode } = require("../src/vscode.ts");
const { Zed } = require("../src/zed.ts");
const { FileSearch } = require("../src/fs.ts");
const { Everything } = require("../src/everything.ts");
const history = ["vscode-remote://ssh-remote+host/alpha", "vscode-remote://ssh-remote+host/beta"];
function editor(Type) {
  const plugin = new Type();
  plugin.files = async () => history;
  return plugin;
}

async function main() {
  assert.equal((await editor(VSCode).enter()).length, 2);
  assert.equal(input, "");
  await editor(VSCode).search("alpha");
  await editor(Zed).search("beta");
  await new FileSearch().search("soil rain");
  const vscode = editor(VSCode);
  assert.equal((await vscode.enter())[0].title, "alpha");
  assert.equal(input, "alpha");
  assert.equal((await editor(Zed).enter())[0].title, "beta");
  assert.equal(input, "beta");
  assert.equal((await new FileSearch().enter())[0].title, "soil rain.pdf");
  assert.equal(input, "soil rain");
  assert.equal(query, "soil rain");

  vscode.isCtrl = true;
  await vscode.select(new ListItem("alpha", history[0], history[0]));
  assert.equal(storage.get("search:vsc_action"), "alpha");
  await vscode.search("");
  await editor(VSCode).enter();
  assert.equal(input, "");
  assert.equal(storage.get("search:zed_action"), "beta");
  await new Everything().search("notes");
  assert.equal(storage.get("search:es"), "notes");
  console.log("PASS 独立关键词、重新进入恢复、输入框同步、清空及收藏保留搜索");
}
main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const Module = require("node:module");
const { transformSync } = require("esbuild");

const commands = [];
const notifications = [];
const fileCommands = [];
const load = Module._load;
Module._load = function (id, ...args) {
  if (id === "utools-helper") return { Setting: { Get: () => "" } };
  if (id === "child_process") return {
    execFile: (bin, args, callback) => {
      fileCommands.push([bin, args]);
      callback(new Error("test failure"));
    },
    exec: (cmd, callback) => {
      commands.push(cmd);
      if (callback) callback(new Error("test failure"));
    },
  };
  return load.call(this, id, ...args);
};
require.extensions[".ts"] = (module, filename) => {
  const { code } = transformSync(fs.readFileSync(filename, "utf8"), {
    loader: "ts", format: "cjs", target: "node16",
  });
  module._compile(code, filename);
};
global.utools = { showNotification: message => notifications.push(message) };

const { editorCmd, quoteArg } = require("../src/plugin.ts");
const target = "/tmp/中文 project's $HOME";
assert.equal(editorCmd("code", target, "zsh -l -c", "darwin"),
  `/usr/bin/open -a 'Visual Studio Code' -- ${quoteArg(target, "darwin")}`);
assert.equal(editorCmd("code-insiders", target, "", "darwin"),
  `/usr/bin/open -a 'Visual Studio Code - Insiders' -- ${quoteArg(target, "darwin")}`);
assert.equal(editorCmd("zed", target, "zsh -l -c", "darwin"),
  `/usr/bin/open -a 'Zed' -- ${quoteArg(target, "darwin")}`);
assert.equal(editorCmd("/custom/code", "/tmp/project", "", "darwin"),
  "/custom/code '/tmp/project'");
assert.equal(editorCmd("code", "vscode-remote://ssh-remote+host/project", "", "darwin"),
  "code --folder-uri 'vscode-remote://ssh-remote+host/project'");
assert.equal(editorCmd("code", "/tmp/project", "", "linux"), "code '/tmp/project'");
assert.equal(editorCmd("code", "C:/project", "", "win32"), 'code "C:/project"');

if (process.platform === "darwin") {
  const { onContextmenu } = require("../src/contextmenu.ts");
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "utools-open-"));
  try {
    const actions = ["explorer", "vscode", "zed", "terminal"].map(action => ({
      style: {}, getAttribute: () => action,
    }));
    const menu = { style: {}, querySelectorAll: () => actions };
    global.document = { querySelector: () => menu };
    onContextmenu({
      target: { closest: () => ({ querySelector: () => ({ innerText: temp }) }) },
      preventDefault() {}, pageX: 0, pageY: 0,
    });
    actions.forEach(action => action.onclick());
    assert.deepEqual(commands, [
      `/usr/bin/open ${quoteArg(temp)}`,
      `/usr/bin/open -a 'Visual Studio Code' -- ${quoteArg(temp)}`,
      `/usr/bin/open -a 'Zed' -- ${quoteArg(temp)}`,
    ]);
    assert.deepEqual(fileCommands, [
      ["/usr/bin/open", ["-na", "kitty", "--args", "--directory", temp]],
    ]);
    assert.equal(notifications.length, 4);
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
}
console.log("PASS 打开命令、平台兼容、右键菜单及错误提示（未打开界面）");

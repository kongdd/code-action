import { readdirSync } from "fs";
import { join } from "path";
import * as util from "util";
import { exec } from "child_process";

export function getIcon(ext: string): string {
  let icons = readdirSync(join(__dirname, "..", "icon"));
  let icon = icons.find((icon) => {
    return "." + icon.split(".")[0] === ext.toLowerCase();
  });
  if (!icon && !ext) icon = "folder.svg";
  if (!icon && ext) icon = "file.svg";
  return join("icon", icon);
}

export function check_path(input: string, platform = process.platform): string {
  if (/vscode-remote:/i.test(input)) return input;

  let p = input;
  if (/^file:/i.test(p)) {
    try { p = decodeURIComponent(p); } catch { /* 非法转义保持原样 */ }
    p = p.replace(/^file:\/\/(?:localhost)?/i, "");
    if (/^\/+[a-zA-Z]:/.test(p)) p = p.replace(/^\/+/, "");
  }
  return platform === "win32" ? p.replace(/\\/g, "/") : p;
}

export function quoteArg(s: string, platform = process.platform): string {
  return platform === "win32"
    ? `"${s.replace(/"/g, '\\"')}"`
    : `'${s.replace(/'/g, `'\\''`)}'`;
}

export function darwinTerminalScript(dir: string): string {
  const sh = `'${dir.replace(/'/g, `'\\''`)}'`;
  const esc = sh.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
  return `tell application "Terminal" to do script "cd ${esc} && clear"`;
}

export function shellWrap(shell: string, cmd: string): string {
  shell = (shell || "").trim();
  return shell ? `${shell} ${quoteArg(cmd, "linux")}` : cmd;
}

export function editorCmd(bin: string, target: string, shell = "", platform = process.platform): string {
  bin = (bin || "code").trim();
  const arg = quoteArg(target, platform);
  const remote = /vscode-remote:/i.test(target);
  // macOS 本地项目通过 LaunchServices 打开，不依赖命令的 PATH。
  if (platform === "darwin" && !remote && !target.startsWith("-") && ["code", "code-insiders", "zed"].includes(bin)) {
    const app = bin === "zed" ? "Zed" : bin === "code" ? "Visual Studio Code" : "Visual Studio Code - Insiders";
    return `/usr/bin/open -a ${quoteArg(app, platform)} -- ${arg}`;
  }
  const flag = target.includes(".code-workspace") ? "--file-uri" : "--folder-uri";
  const qbin = /[\s"]/.test(bin) ? quoteArg(bin, platform) : bin;
  const cmd = remote ? `${qbin} ${flag} ${arg}` : `${qbin} ${arg}`;
  return shellWrap(shell, cmd);
}

// Promisify the exec function
export const execPromise = util.promisify(exec);

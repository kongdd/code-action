import { join } from "path";
import { platform } from "process";
import { IConfigItem } from "utools-helper";

let defaultShell = "bash -l -c";
let defaultTerminal = "";
let defaultExplorer = "";

switch (platform) {
  case "win32":
    defaultShell = "";
    defaultTerminal = "C:/Users/" + process.env.USERNAME + "/AppData/Local/Microsoft/WindowsApps/wt.exe -d";
    defaultExplorer = "explorer.exe";
    break;
  case "darwin":
    defaultShell = "zsh -l -c";
    defaultTerminal = "";
    defaultExplorer = "open";
    break;
  case "linux":
    defaultShell = "bash -l -c";
    defaultTerminal = "gnome-terminal --working-directory";
    defaultExplorer = "xdg-open";
    break;
}

export const config: IConfigItem[] = [
  {
    name: "shell",
    label: "shell",
    type: "input",
    required: false,
    placeholder: "一般情况下无需修改，windows 请保持为空值",
    default: defaultShell,
    only_current_machine: true,
  },
  {
    name: "code",
    label: "code",
    type: "input",
    placeholder: "vscode 命令",
    required: true,
    default: "code",
    only_current_machine: true,
  },
  {
    name: "db",
    label: "db",
    type: "input",
    required: true,
    only_current_machine: true,
    default: join(
      utools.getPath("appData"),
      "Code",
      "User",
      "globalStorage",
      "state.vscdb"
    ),
  },
  {
    name: "storage",
    label: "storage",
    type: "input",
    required: true,
    only_current_machine: true,
      default: join(utools.getPath("appData"), "Code", "User", "globalStorage", "storage.json"),
  },
  {
    name: "timeout",
    label: "timeout",
    type: "input",
    placeholder: "shell 命令执行超时时间",
    required: false,
    default: "3000",
    only_current_machine: true,
  },
  {
    name: "terminal",
    label: "terminal",
    type: "input",
    placeholder: "留空：Windows wt.exe，macOS Terminal.app，Linux gnome-terminal",
    required: false,
    default: defaultTerminal,
    only_current_machine: true,
  },
  {
    name: "explorer",
    label: "explorer",
    type: "input",
    placeholder: "文件管理器命令 (Windows: explorer.exe, macOS/Linux: open/xdg-open)",
    required: false,
    default: defaultExplorer,
    only_current_machine: true,
  },
];

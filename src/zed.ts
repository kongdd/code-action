import { VSCode } from "./vscode";
import { check_path } from "./plugin";
import { Setting } from "utools-helper";
import { ListItem } from "utools-helper";

export class Zed extends VSCode {
  code = "zed_action";

  async select(item: ListItem<string>) {
    // Ctrl 键：切换收藏状态
    if (this.isCtrl) {
      return await super.select(item);
    }

    // 使用 zed 打开，description 是未编码的原始路径
    const path = check_path(item.description);
    let cmd = `zed "${path}"`;

    const shell = Setting.Get("shell");
    if (shell.trim()) cmd = `${shell} "${cmd}"`;

    console.log(cmd);

    const timeout = Math.max(parseInt(Setting.Get("timeout")) || 3000, 3000);

    this.execCmd(cmd, { timeout, windowsHide: true, encoding: "utf-8" })
      .catch((reason) => { throw reason.toString(); });
  }
}

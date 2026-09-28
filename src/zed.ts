import { VSCode } from "./vscode";
import { check_path, quoteArg, shellWrap } from "./plugin";
import { Setting } from "utools-helper";
import { ListItem } from "utools-helper";

export class Zed extends VSCode {
  code = "zed_action";

  async select(item: ListItem<string>) {
    // Ctrl 键：切换收藏状态
    if (this.isCtrl) {
      return await super.select(item);
    }

    const target = check_path(item.description);
    const cmd = shellWrap(Setting.Get("shell") || "", `zed ${quoteArg(target)}`);

    console.log(cmd);

    const timeout = Math.max(parseInt(Setting.Get("timeout")) || 3000, 3000);

    this.execCmd(cmd, { timeout, windowsHide: true, encoding: "utf-8" })
      .catch((reason) => { throw reason.toString(); });
  }
}

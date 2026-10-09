import { execFile } from "child_process";
import { existsSync } from "fs";
import { homedir } from "os";
import { basename, extname, join } from "path";
import { promisify } from "util";
import { ListItem, Plugin } from "utools-helper";
import { getIcon } from "./plugin";

const locate = promisify(execFile);
// uTools 的 PATH 未必包含 Cargo 安装目录。
const cargoBinary = join(homedir(), ".cargo", "bin", process.platform === "win32" ? "fs.exe" : "fs");
const binary = existsSync(cargoBinary) ? cargoBinary : "fs";

export class FileSearch implements Plugin {
  code = "fs_action";
  delay = 200;
  placeholder = "搜索全部索引，例如 *.pdf、soil rain、path:papers Kong";

  async enter(): Promise<ListItem[]> {
    return [];
  }

  async search(word = ""): Promise<ListItem[]> {
    if (!word.trim()) return [];

    try {
      // 直接传参，保留表达式中的引号、通配符和特殊字符，不经 shell。
      const { stdout } = await locate(binary,
        ["locate", "--json", "-n", "200", "--", word], {
          encoding: "utf8",
          timeout: 3000,
          maxBuffer: 10 * 1024 * 1024,
          windowsHide: true,
        });
      const files: { path: string }[] = JSON.parse(stdout);
      return files.map(({ path }) => new ListItem<string>(
        basename(path), path, path, getIcon(extname(path))
      ));
    } catch (error) {
      console.error(error);
      return [new ListItem("fs 搜索失败", String(error.stderr || error.message || error), "")];
    }
  }

  select(item: ListItem<string>) {
    if (!item.data) return;
    utools.shellOpenPath(item.data);
    utools.hideMainWindow();
  }
}

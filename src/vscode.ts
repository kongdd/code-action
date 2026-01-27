import { Plugin, ListItem, Setting } from "utools-helper";
import { basename, extname } from "path";
import { existsSync } from "fs";
import { ExecOptions, exec } from "child_process";
import { GetFiles } from "./files";
import { getIcon, check_path } from "./plugin";
import { InitContextMenu, onContextmenu } from "./contextmenu";


export class VSCode implements Plugin {
  code = "vsc_action";
  _storage: string;
  isCtrl = false;
  delay = 100;

  constructor() {
    // 在父元素上监听所有 .list-item 的点击
    document.addEventListener('DOMContentLoaded', InitContextMenu);
    document.onkeydown = (ev) => {
      if (ev.ctrlKey || ev.metaKey) {
        this.isCtrl = true;
      }
    };
    document.oncontextmenu = onContextmenu;
  }

  async files() {
    try {
      return await GetFiles(this.storage);
    } catch (error) {
      console.error(error);
      return [];
    }
  }

  get storage(): string {
    if (!this._storage) this._storage = Setting.Get("db");
    return this._storage;
  }

  async enter(): Promise<ListItem[]> {
    return await this.search("");
  }

  async search(word?: string): Promise<ListItem[]> {
    this.isCtrl = false;
    let files = await this.files();

    // 多关键词过滤
    const keywords = word.split(/\s+/g).filter(k => k.trim());
    if (keywords.length > 0) {
      files = files.filter((file: string) => {
        const lowerFile = decodeURIComponent(file).toLowerCase();
        return keywords.every(k => lowerFile.includes(k.toLowerCase()));
      });
    }

    // 转换为 ListItem 并过滤不存在的本地路径
    let items = files
      .map((file: any): ListItem => {
        const decodedFile = decodeURIComponent(file);
        const normalizedPath = check_path(decodedFile);
        const isRemote = normalizedPath.includes("vscode-remote") || normalizedPath.includes("remote");

        return new ListItem<string>(
          basename(normalizedPath),
          normalizedPath,
          isRemote ? normalizedPath : encodeURIComponent(normalizedPath),
          getIcon(isRemote ? ".remote" : extname(normalizedPath))
        );
      })
      .filter(item => {
        // remote 项目跳过路径检查，本地项目检查 description（未编码的路径）
        const isRemote = item.description.includes("vscode-remote") || item.description.includes("remote");
        return isRemote || existsSync(item.description);
      });

    // 无搜索词时，收藏项目显示在最前面
    if (keywords.length === 0) {
      const collects = this.getCollect().map(item => {
        item.icon = "icon/icon-collect.png";
        return item;
      });

      // 去重：移除已在收藏列表中的项目
      const collectPaths = new Set(collects.map(c => c.description));
      items = collects.concat(items.filter(item => !collectPaths.has(item.description)));
    }

    return items;
  }

  getCollect(): ListItem[] {
    return utools.dbStorage.getItem("collect") || [];
  }

  saveCollect(item: ListItem) {
    let items = this.getCollect();
    item.icon = "icon/icon-collect.png";
    items.unshift(item);
    utools.dbStorage.setItem("collect", items);

    utools.showNotification(`${item.title} 已置顶`);
  }

  removeCollect(item: ListItem) {
    let items = this.getCollect();
    items = items.filter((data) => data.description != item.description);
    utools.dbStorage.setItem("collect", items);
    utools.showNotification(`${item.title} 置顶已移除`);
  }

  private execCmd(command: string, options: { encoding: BufferEncoding } & ExecOptions): Promise<string> {
    return new Promise((resolve, reject) => {
      exec(command, options, (_, stdout, stderr) => {
        stderr ? reject(stderr) : resolve(stdout);
      });
    });
  }

  async select(item: ListItem<string>) {
    // Ctrl 键：切换收藏状态
    if (this.isCtrl) {
      const collects = this.getCollect();
      const isSaved = collects.some(i => i.description === item.description);
      isSaved ? this.removeCollect(item) : this.saveCollect(item);
      this.isCtrl = false;
      return await this.search("");
    }

    // 构建打开命令
    let code = Setting.Get("code");
    if (code.includes(" ")) code = `"${code}"`;

    const uriFlag = item.data.includes(".code-workspace") ? "--file-uri" : "--folder-uri";
    const path = check_path(item.data);
    let cmd = `${code} ${uriFlag} "${path}"`;

    const shell = Setting.Get("shell");
    if (shell.trim()) cmd = `${shell} "${cmd}"`;

    console.log(cmd);

    const timeout = Math.max(parseInt(Setting.Get("timeout")) || 3000, 3000);

    this.execCmd(cmd, { timeout, windowsHide: true, encoding: "utf-8" })
      .catch((reason) => { throw reason.toString(); });
  }
}

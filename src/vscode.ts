import { ListItem, Setting } from "utools-helper";
import { SearchPlugin } from "./search";
import { basename, extname } from "path";
import { existsSync } from "fs";
import { ExecOptions, exec } from "child_process";
import { GetFiles } from "./files";
import { getIcon, check_path, editorCmd } from "./plugin";
import { InitContextMenu, onContextmenu } from "./contextmenu";


export class VSCode extends SearchPlugin {
  code = "vsc_action";
  _storage: string;
  isCtrl = false;
  delay = 100;

  constructor() {
    super();
    // 在父元素上监听所有 .list-item 的点击
    document.addEventListener('DOMContentLoaded', InitContextMenu);
    document.onkeydown = (ev) => {
      if (ev.ctrlKey || ev.metaKey) this.isCtrl = true;

      if (["ArrowDown", "PageDown", "PageUp"].includes(ev.code)) {
        ev.preventDefault();
        ev.stopPropagation();
        this.move(ev.code === "PageUp" ? "ArrowUp" : "ArrowDown", ev.code === "ArrowDown" ? 1 : 10);
      }
    };
    document.oncontextmenu = onContextmenu;
  }

  private loadNextPage(): boolean {
    const marker = document.querySelector(".list-item-selected + span");
    const list = marker?.closest<HTMLElement>(".list");
    if (!list) return false;

    list.scrollTop++;
    list.dispatchEvent(new Event("scroll"));
    return true;
  }

  private move(code: "ArrowDown" | "ArrowUp", remaining: number) {
    if (code === "ArrowDown" && this.loadNextPage()) {
      requestAnimationFrame(() => this.move(code, remaining));
      return;
    }

    window.dispatchEvent(new KeyboardEvent("keydown", { code, key: code }));
    if (remaining > 1) requestAnimationFrame(() => this.move(code, remaining - 1));
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

  async search(word = ""): Promise<ListItem[]> {
    this.remember(word);
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
        const isRemote = /vscode-remote:/i.test(normalizedPath);

        return new ListItem<string>(
          basename(normalizedPath),
          normalizedPath,
          normalizedPath,
          getIcon(isRemote ? ".remote" : extname(normalizedPath))
        );
      })
      .filter(item => {
        // remote 项目跳过路径检查，本地项目检查 description（未编码的路径）
        const isRemote = /vscode-remote:/i.test(item.description);
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

  protected execCmd(command: string, options: { encoding: BufferEncoding } & ExecOptions): Promise<string> {
    return new Promise((resolve, reject) => {
      exec(command, options, (err, stdout, stderr) => {
        err ? reject(stderr || err.message) : resolve(stdout);
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
      return await this.search(this.keyword);
    }

    const cmd = editorCmd(Setting.Get("code"), check_path(item.description), Setting.Get("shell") || "");
    console.log(cmd);

    const timeout = Math.max(parseInt(Setting.Get("timeout")) || 3000, 3000);

    this.execCmd(cmd, { timeout, windowsHide: true, encoding: "utf-8" })
      .catch((reason) => { throw reason.toString(); });
  }
}

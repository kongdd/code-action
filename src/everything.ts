import { Plugin, ListItem, Setting } from "utools-helper";
import { basename, join, extname } from "path";
import { readdirSync, readFileSync } from "fs";
import { ExecOptions, exec } from "child_process";
// import { GetFiles } from "./files";
import { getIcon, execPromise } from "./plugin";
// import { exec } from "child_process";
import * as path from "path";
import * as util from "util";

import {
  // handleContextAction,
  // setupContextMenu,
  InitContextMenu,
  onContextmenu,
  // insertCSS,
  // exec
} from "./contextmenu";


export async function runCommand(cmd: string, options:any) {
  try {
    const { stdout, stderr } = await execPromise(cmd, options);
    // console.log(`stdout: ${stdout}`);
    if (stderr) console.error(`stderr: ${stderr}`);
    return stdout;
  } catch (error) {
    console.error(`Execution error: ${error}`);
    throw error; // Or handle it as needed
  }
}

var ext_exclude = [".lnk", ".log"];

async function GetFiles(pattern: string) {
  pattern = pattern || "soil moisture";
  // const app = path.resolve(__dirname, "../bin/es.exe");
  const app = "C:/Program Files/Everything/es.exe";
  // alert(app);
  // const pattern = "soil moisture";
  const cmd = `"${app}" -n 200 /o-d "${pattern}"`; // e.g., "C:\path\to\es.exe" "soil moisture"
  const maxBuffer = 1024 * 1024 * 10; // 1MB buffer
  const result = await runCommand(cmd, { maxBuffer: maxBuffer });
  const filePaths = result.toString().split('\r\n').filter(line => line.trim() !== '');
  
  return filePaths.filter((file) => {
    const ext = path.extname(file).toLowerCase();
    return !ext_exclude.includes(ext);
  });
  // return filePaths;
}

export class Everything implements Plugin {
  code = "es";
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

  async files(pattern:string) {
    try {
      // let pattern = "soil moisture";
      return await GetFiles(pattern);
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
    if (process.platform !== "win32") {
      return [new ListItem("Everything 仅支持 Windows", "macOS / Linux 不可用", "")];
    }
    let pattern = word || "";
    
    this.isCtrl = false;
    let files = await this.files(pattern);
    if (!files) return [];
    // 搜索
    // word.split(/\s+/g).forEach((keyword) => {
    //   files = files.filter((file: string) => {
    //     return decodeURIComponent(file)
    //       .toLowerCase()
    //       .includes(keyword.trim().toLowerCase());
    //   });
    // });

    let items = files.map(
      (file: any): ListItem => {
        let address = file;
        // file = decodeURIComponent(file);
        let item = new ListItem<string>(basename(file), file, address);
        let ext = file.includes("remote") ? ".remote" : extname(file);
        item.icon = getIcon(ext);
        return item;
      }
    );

    if (!word.trim()) {
      let collects = this.getCollect();

      collects.map((item) => (item.icon = "icon/icon-collect.png"));

      // 去除已收藏的项目，避免重复显示
      items = items.filter((item) => {
        for (let i = 0; i < collects.length; i++) {
          if (item.description == collects[i].description) return false;
        }
        return true;
      });

      items = collects.concat(items);
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

  private execCmd(
    command: string,
    options: { encoding: BufferEncoding } & ExecOptions
  ): Promise<string> {
    return new Promise((resolve, reject) => {
      exec(command, options, (_, stdout, stderr) => {
        if (stderr) return reject(stderr);

        resolve(stdout);
      });
    });
  }

  async select(item: ListItem<string>) {
    if (this.isCtrl) {
      let items = this.getCollect();
      let isSave = items.find((i) => i.description == item.description);
      if (isSave) this.removeCollect(item);
      else this.saveCollect(item);
      this.isCtrl = false;
      return await this.search("");
    }
    console.log(item);
    var path = item.data;
    if (process.platform === 'win32') {
      path = path.replace(/\//g, '\\');
    }

    let ext = path.includes("remote") ? "" : extname(path);    
    var app = ext == "" ? "explorer.exe" : "";
    var cmd = `${app} "${path}"`;
    try {
      await runCommand(cmd, {});
    } catch (error) {
      // ignore app error
      console.error(error);
    }
    

    // let code = Setting.Get("code");
    // if (code.trim().includes(" ")) code = `"${code}"`;

    // let cmds: String[] = [code];
    // if (item.data.includes(".code-workspace")) cmds.push("--file-uri");
    // else cmds.push("--folder-uri");

    // cmds.push(`"${item.data}"`);

    // let cmd = cmds.join(" ");
    // let shell = Setting.Get("shell");
    // if (shell.trim()) cmd = `${shell} "${cmd}"`;
    // console.log(cmd);

    // let timeout = parseInt(Setting.Get("timeout"));
    // if (!timeout || timeout < 3000) timeout = 3000;

    // // console.log(cmd);
    // this.execCmd(cmd, {
    //   timeout: timeout, windowsHide: true,
    //   encoding: "utf-8"
    // })
    //   // .then(() => utools.hideMainWindow())
    //   .catch((reason) => { throw reason.toString() });
  }
}

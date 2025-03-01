import { readFileSync, existsSync } from "fs";
// import initSqlJs from "sql.js";
import { Database } from "sql.js";
const initSqlJs = require("../third_party/sqljs/sql-wasm");

export interface Recent {
  entries: Entry[];
}

export interface Entry {
  folderUri?: string;
  workspace?: Workspace;
  label?: string;
  remoteAuthority?: string;
  fileUri?: string;
}

export interface Workspace {
  id: string;
  configPath: string;
}

export async function GetFiles(path: string) {
  let config = {
    locateFile: (file: String) => `node_modules/sql.js/dist/${file}`
    // locateFile: file => `../third_party/sqljs/${file}`
  };
  // const SQL = await initSqlJs(config);
  const SQL = await initSqlJs();
  let db = new SQL.Database(readFileSync(path)) as Database;
  let sql =
    "select value from ItemTable where key = 'history.recentlyOpenedPathsList'";
  let results = db.exec(sql);
  let res = results[0].values.toString();
  if (!res)
    throw new Error(
      "数据获取失败, 请检查 vsc-setting 配置, <br/> 注意当前仅在 vscode 1.64 版本进行过测试"
    );
  let data = JSON.parse(res) as Recent;
  
  // 只检查前20个
  let paths = data.entries.map((file) => {
    var path = "";
    if (typeof file === "string") {
      path = file;
    } else {
      path = file.fileUri || file.folderUri || file.workspace.configPath;
    }
    return path;
  });
  
  if (paths.length <= 20) {
    return paths.map((path: String) => {
      const _path = decodeURIComponent(path.replace("file:///", ""));
      const fileExists = existsSync(_path);
      return { path, fileExists };
    })
      .filter((item) => item.fileExists) // 过滤掉不存在的文件
      .map((item) => item.path); // 最后只保留路径    
  }
  return paths;
}

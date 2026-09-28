import { existsSync, readFileSync } from "fs";
import { join } from "path";
import { homedir } from "os";
import { Database } from "sql.js";
// 为了减少文件大小，从最新的"sql.js" copy出来
const initSqlJs = require("../third_party/sqljs/sql-wasm");

const STORAGE_KEY = "history.recentlyOpenedPathsList";

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

// VSCode 1.118+ 将历史迁移到此路径，需优先读取
function getSharedStoragePath(): string {
  const home = homedir();
  return home ? join(home, ".vscode-shared", "sharedStorage", "state.vscdb") : "";
}

// 读取数据库文件，自动合并同名 .wal 文件（VSCode 开启 WAL 模式后未 checkpoint 的数据在此）
function readDbBuffer(dbPath: string): Buffer {
  const db = readFileSync(dbPath);
  const walPath = `${dbPath}-wal`;
  if (!existsSync(walPath)) return db;

  const wal = readFileSync(walPath);
  if (wal.length < 32) return db;
  if (wal.readUInt32BE(0) !== 0x377f0682 && wal.readUInt32BE(0) !== 0x377f0683) return db;

  const pageSize = wal.readUInt32BE(8) || (db.length >= 100 ? db.readUInt16BE(16) || 4096 : 4096);
  const [salt1, salt2] = [wal.readUInt32BE(16), wal.readUInt32BE(20)];
  const frameSize = pageSize + 24;
  let merged = new Uint8Array(db);
  let committedPageCount = Math.ceil(merged.length / pageSize);
  let pending: Array<{ pageNumber: number; page: Uint8Array }> = [];

  for (let offset = 32; offset + frameSize <= wal.length; offset += frameSize) {
    const pageNumber = wal.readUInt32BE(offset);
    const commitCount = wal.readUInt32BE(offset + 4);
    if (!pageNumber || wal.readUInt32BE(offset + 8) !== salt1 || wal.readUInt32BE(offset + 12) !== salt2) continue;

    pending.push({ pageNumber, page: new Uint8Array(wal.buffer, wal.byteOffset + offset + 24, pageSize) });

    if (commitCount) {
      for (const { pageNumber, page } of pending) {
        const pageOffset = (pageNumber - 1) * pageSize;
        if (merged.length < pageOffset + pageSize) {
          const expanded = new Uint8Array(pageOffset + pageSize);
          expanded.set(merged);
          merged = expanded;
        }
        merged.set(page, pageOffset);
      }
      committedPageCount = commitCount;
      pending = [];
    }
  }

  return Buffer.from(merged.subarray(0, committedPageCount * pageSize));
}

export async function GetFiles(dbPath: string): Promise<string[]> {
  const SQL = await initSqlJs();

  // VSCode 1.118+: 优先使用 shared storage 路径，旧路径作为 fallback
  const sharedPath = getSharedStoragePath();
  const isDefaultVscPath = dbPath.replace(/\\/g, "/").toLowerCase().endsWith("/code/user/globalstorage/state.vscdb");
  const candidates = (sharedPath && existsSync(sharedPath) && isDefaultVscPath)
    ? [sharedPath, dbPath]
    : [dbPath];

  let lastError: Error;
  for (const path of candidates) {
    let db: Database | undefined;
    try {
      db = new SQL.Database(readDbBuffer(path)) as Database;
      const results = db.exec(`select value from ItemTable where key = '${STORAGE_KEY}'`);
      const value = results?.[0]?.values?.[0]?.[0];
      if (!value) throw new Error("未找到历史记录数据");
      const data = JSON.parse(value.toString()) as Recent;
      return data.entries.map(file => {
        if (typeof file === "string") return file;
        return file.fileUri || file.folderUri || file.workspace?.configPath || "";
      });
    } catch (e) {
      lastError = e as Error;
    } finally {
      db?.close();
    }
  }

  throw new Error(`${lastError.message}。请检查 vsc-setting 配置；VS Code 1.118+ 需要使用 .vscode-shared/sharedStorage/state.vscdb`);
}


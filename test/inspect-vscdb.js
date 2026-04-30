const initSqlJs = require("../third_party/sqljs/sql-wasm");
const { readFileSync, existsSync } = require("fs");
const path = require("path");

// VSCode 1.118+ 新路径，旧路径注释切换
const dbPath = path.join(process.env.USERPROFILE, ".vscode-shared/sharedStorage/state.vscdb");
// const dbPath = path.join(process.env.APPDATA, "Code/User/globalStorage/state.vscdb");

// 合并 WAL 后读取数据库 buffer（复制自 dist/files.js 的逻辑）
function readDbBuffer(dbPath) {
  const db = readFileSync(dbPath);
  const walPath = `${dbPath}-wal`;
  if (!existsSync(walPath)) return db;
  const wal = readFileSync(walPath);
  if (wal.length < 32) return db;
  const magic = wal.readUInt32BE(0);
  if (magic !== 0x377f0682 && magic !== 0x377f0683) return db;
  const pageSize = wal.readUInt32BE(8) || (db.length >= 100 ? db.readUInt16BE(16) || 4096 : 4096);
  const [salt1, salt2] = [wal.readUInt32BE(16), wal.readUInt32BE(20)];
  const frameSize = pageSize + 24;
  let merged = new Uint8Array(db);
  let committedPageCount = Math.ceil(merged.length / pageSize);
  let pending = [];
  for (let offset = 32; offset + frameSize <= wal.length; offset += frameSize) {
    const pageNumber = wal.readUInt32BE(offset);
    const commitCount = wal.readUInt32BE(offset + 4);
    if (!pageNumber || wal.readUInt32BE(offset + 8) !== salt1 || wal.readUInt32BE(offset + 12) !== salt2) continue;
    pending.push({ pageNumber, page: new Uint8Array(wal.buffer, wal.byteOffset + offset + 24, pageSize) });
    if (commitCount) {
      for (const { pageNumber, page } of pending) {
        const pageOffset = (pageNumber - 1) * pageSize;
        if (merged.length < pageOffset + pageSize) { const e = new Uint8Array(pageOffset + pageSize); e.set(merged); merged = e; }
        merged.set(page, pageOffset);
      }
      committedPageCount = commitCount;
      pending = [];
    }
  }
  return Buffer.from(merged.subarray(0, committedPageCount * pageSize));
}

(async () => {
  if (!existsSync(dbPath)) { console.error("数据库文件不存在:", dbPath); return; }

  const SQL = await initSqlJs();
  const db = new SQL.Database(readDbBuffer(dbPath));

  console.log("=== 所有表 ===");
  db.exec("SELECT name FROM sqlite_master WHERE type='table'")
    [0]?.values.forEach(([name]) => console.log(name));

  console.log("\n=== 含 recent/history/opened 的键 ===");
  db.exec("SELECT key, length(value) FROM ItemTable WHERE key LIKE '%recent%' OR key LIKE '%history%' OR key LIKE '%opened%'")
    [0]?.values.forEach(([k, v]) => console.log(String(k).padEnd(50), v));

  console.log("\n=== 最大的前10条记录 ===");
  db.exec("SELECT key, length(value) FROM ItemTable ORDER BY length(value) DESC LIMIT 10")
    [0]?.values.forEach(([k, v]) => console.log(String(k).padEnd(50), v));

  for (const key of ["recently.opened", "history.recentlyOpenedPathsList"]) {
    const res = db.exec(`SELECT value FROM ItemTable WHERE key = '${key}'`);
    const data = res[0] ? JSON.parse(res[0].values[0][0]) : null;
    console.log(`\n=== ${key} ===`);
    console.log("条目数:", data?.entries?.length ?? "未找到");
    if (data?.entries?.length) console.log("前3条:", JSON.stringify(data.entries.slice(0, 3), null, 2));
  }

  db.close();
})().catch(console.error);


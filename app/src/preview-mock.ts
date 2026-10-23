// 浏览器预览用的 IPC 模拟，仅在 index.html?preview 时加载。
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";

const items = [
  { title: "hello.html", path: "/mnt/z/backup_N/Research/面试/interview/hello.html", remote: false },
  { title: "hello.md", path: "/mnt/z/backup_N/Research/面试/interview/hello.md", remote: false },
  { title: "hello.pdf", path: "/mnt/z/backup_N/Research/面试/interview/hello.pdf", remote: false },
  { title: "hello.pptx", path: "/mnt/z/backup_N/Research/面试/interview/hello.pptx", remote: false },
  { title: "ex-hello.R", path: "/mnt/z/backup_N/Research/rpkgs_cran/sp/ex-hello.R", remote: false },
  { title: "hello", path: "/mnt/z/backup_N/Research/ML/hello", remote: false },
  { title: "hello.Rproj", path: "/mnt/z/backup_N/Research/ML/hello/hello.Rproj", remote: false },
  { title: "hello.R", path: "/mnt/z/backup_N/Research/ML/hello/R/hello.R", remote: false },
  { title: "hello.jl", path: "/mnt/z/backup_N/MODIS/hello.jl", remote: false },
  { title: "utools-vscode", path: "vscode-remote://ssh-remote+box/home/kongdd/utools-vscode", remote: true },
];

mockWindows("main");
mockIPC((cmd) => {
  if (cmd === "load_store") return {
    settings: {
      database: "",
      fsBinary: "/Users/kongdd/.cargo/bin/fs",
      code: "code",
      zed: "zed",
      terminal: "",
      shortcuts: [
        { id: "toggle", accelerator: "CommandOrControl+Shift+Space", action: "toggle", enabled: true, autoCopy: false },
        { id: "vscode", accelerator: "Command+Control+C", action: "vscode", enabled: true, autoCopy: false },
        { id: "zed", accelerator: "Command+Control+Z", action: "zed", enabled: true, autoCopy: false },
        { id: "fs", accelerator: "Command+Control+E", action: "fs", enabled: true, autoCopy: true },
      ],
      aliases: {},
    },
    preferences: { favorites: [], queries: {}, mode: "fs" },
  };
  if (cmd === "search") return items;
  if (cmd === "plugin:event|listen") return 1;
  return null;
});

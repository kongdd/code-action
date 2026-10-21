import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./style.css";

type Mode = "vscode" | "zed" | "fs";
type Action = "default" | "vscode" | "insiders" | "zed" | "directory" | "terminal";
interface Item { title: string; path: string; remote: boolean }
interface Shortcut {
  id: string;
  accelerator: string;
  action: string;
  enabled: boolean;
  autoCopy: boolean;
}
interface Settings {
  database: string;
  fsBinary: string;
  code: string;
  insiders: string;
  zed: string;
  terminal: string;
  shortcuts: Shortcut[];
  aliases: Record<string, string>;
}
interface Preferences { favorites: string[]; queries: Record<string, string>; mode: string }
interface Store { settings: Settings; preferences: Preferences }

const MODES: Record<Mode, string> = {
  vscode: "vsc_action",
  zed: "zed_action",
  fs: "fs_action",
};
const ACTIONS: [string, string][] = [
  ["toggle", "显示/隐藏窗口"],
  ["vscode", "vsc_action"],
  ["zed", "zed_action"],
  ["fs", "fs_action"],
];

function element<T extends HTMLElement>(selector: string): T {
  const node = document.querySelector<T>(selector);
  if (!node) throw new Error(`Missing element: ${selector}`);
  return node;
}

const query = element<HTMLInputElement>("#query");
const results = element<HTMLUListElement>("#results");
const notice = element("#notice");
const empty = element("#empty");
const emptyTitle = element(".empty-title");
const loading = element("#loading");
const limit = element("#limit");
const panel = element("#search-panel");
const settingsPanel = element("#settings");
const generalForm = element<HTMLFormElement>("form.pane");
const menu = element("#context-menu");
const modeToggle = element<HTMLButtonElement>("#mode-toggle");
const modeMenu = element<HTMLUListElement>("#mode-menu");
const shortcutRows = element("#shortcut-rows");
const aliasRows = element("#alias-rows");
let store: Store;
let mode: Mode = "vscode";
let items: Item[] = [];
let selected = 0;
let epoch = 0;
let timer: ReturnType<typeof setTimeout>;
let saveQueue = Promise.resolve();
let opening = false;
let recording: { id: string; element: HTMLElement } | null = null;
let autoCopyNext = false;
let saveTimer: ReturnType<typeof setTimeout>;
let noticeTimer: ReturnType<typeof setTimeout>;

/// 与 Rust 侧 fs locate 的结果上限一致。
const RESULT_LIMIT = 200;

function report(error: unknown) {
  notice.textContent = String(error);
  notice.hidden = false;
  // 错误提示自动退场，避免一条旧错误长期占据顶部。
  clearTimeout(noticeTimer);
  noticeTimer = setTimeout(() => { notice.hidden = true; }, 8000);
}

function persist(): Promise<void> {
  const preferences = structuredClone(store.preferences);
  // 串行写入，避免慢的旧请求覆盖最新的关键字与收藏。
  saveQueue = saveQueue.catch(() => {}).then(() => invoke("save_preferences", { preferences }));
  return saveQueue;
}

function remember() {
  store.preferences.queries[mode] = query.value;
  store.preferences.mode = mode;
  // 防抖写盘：连续键入不必每次都落盘。
  clearTimeout(saveTimer);
  saveTimer = setTimeout(() => void persist().catch(report), 400);
}

/// 隐藏前把未落盘的查询刷到磁盘，然后收起窗口。
function dismiss() {
  clearTimeout(saveTimer);
  autoCopyNext = false;
  void persist().catch(report);
  void invoke("hide_window").catch(report);
}

function active(scroll = false) {
  for (const [i, node] of [...results.children].entries()) {
    const current = i === selected;
    node.setAttribute("aria-selected", String(current));
    const hint = node.querySelector(".shortcut");
    // 右侧提示只标注当前行与可用数字键，避免每行都塞入键位。
    if (hint) hint.textContent = current ? "↵" : i < 9 ? "⌘" + (i + 1) : "";
  }
  const node = results.children[selected];
  if (node) {
    query.setAttribute("aria-activedescendant", node.id);
    if (scroll) node.scrollIntoView({ block: "nearest" });
  } else {
    query.removeAttribute("aria-activedescendant");
  }
}

// 图标与 uTools 保持一致：按扩展名匹配 public/icons 下的同名资源。
const ICONS = ["c", "cpp", "cs", "css", "doc", "docx", "file", "folder", "go", "html",
  "java", "js", "json", "md", "pdf", "php", "pptx", "py", "remote", "sh", "ts",
  "vscode", "vue", "xml", "yaml"];

function iconFor(item: Item): string {
  if (item.remote) return "/icons/remote.svg";
  const name = item.path.split("/").pop() || "";
  const dot = name.lastIndexOf(".");
  const ext = dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
  if (!ext) return "/icons/folder.svg";
  return `/icons/${ICONS.includes(ext) ? ext : "file"}.svg`;
}

/// fs 模式下把查询词还原为可高亮的片段，去掉 path: 等前缀与通配符。
function terms(): string[] {
  if (mode !== "fs") return [];
  return query.value
    .split(/\s+/)
    .map((word) => word.replace(/^(path|name|ext):/i, "").replaceAll("*", ""))
    .filter((word) => word.length > 1);
}

/// 用 mark 包裹命中片段，其余保持纯文本节点。
function fill(node: HTMLElement, text: string, words: string[]) {
  node.replaceChildren();
  if (!words.length) { node.textContent = text; return; }
  const lower = text.toLowerCase();
  const hits: [number, number][] = [];
  for (const word of words) {
    const target = word.toLowerCase();
    for (let at = lower.indexOf(target); at >= 0; at = lower.indexOf(target, at + target.length)) {
      hits.push([at, at + target.length]);
    }
  }
  hits.sort((a, b) => a[0] - b[0]);
  const merged: [number, number][] = [];
  for (const hit of hits) {
    const last = merged.at(-1);
    if (last && hit[0] <= last[1]) last[1] = Math.max(last[1], hit[1]);
    else merged.push([...hit]);
  }
  let cursor = 0;
  for (const [start, end] of merged) {
    if (start > cursor) node.append(text.slice(cursor, start));
    const mark = document.createElement("mark");
    mark.textContent = text.slice(start, end);
    node.append(mark);
    cursor = end;
  }
  if (cursor < text.length) node.append(text.slice(cursor));
}

function render() {
  results.replaceChildren();
  const words = terms();
  items.forEach((item, index) => {
    const row = document.createElement("li");
    row.id = `result-${index}`;
    row.role = "option";
    const icon = document.createElement("img");
    icon.src = iconFor(item);
    icon.alt = "";
    const text = document.createElement("div");
    text.className = "text";
    const title = document.createElement("strong");
    const pinned = store.preferences.favorites.includes(item.path);
    fill(title, (pinned ? "★ " : "") + item.title, words);
    const path = document.createElement("span");
    fill(path, item.path, words);
    text.append(title, path);
    row.append(icon, text);
    const shortcut = document.createElement("span");
    shortcut.className = "shortcut";
    row.append(shortcut);
    row.addEventListener("mousemove", () => { selected = index; active(); });
    row.addEventListener("click", () => void open(item));
    row.addEventListener("contextmenu", (event) => context(event, item));
    results.append(row);
  });
  active();
}

function schedule(delay = 180) {
  clearTimeout(timer);
  const request = ++epoch;
  items = [];
  selected = 0;
  menu.hidden = true;
  render();
  empty.hidden = true;
  notice.hidden = true;
  loading.hidden = false;
  limit.hidden = true;
  timer = setTimeout(() => void search(request), delay);
}

async function search(request: number) {
  try {
    const found = await invoke<Item[]>("search", { mode, query: query.value });
    if (request !== epoch) return;
    items = found;
    selected = 0;
    loading.hidden = true;
    render();
    limit.hidden = !(mode === "fs" && found.length >= RESULT_LIMIT);
    empty.hidden = items.length > 0;
    emptyTitle.textContent = mode === "fs" && !query.value.trim()
      ? "输入关键词搜索全部文件索引" : "没有匹配结果";
  } catch (error) {
    if (request !== epoch) return;
    items = [];
    loading.hidden = true;
    render();
    empty.hidden = true;
    report(error);
  }
}

function setMode(next: Mode) {
  mode = next;
  store.preferences.mode = mode;
  modeToggle.textContent = store.settings.aliases[mode] || MODES[mode];
  for (const option of modeMenu.querySelectorAll<HTMLElement>("li[data-mode]")) {
    option.setAttribute("aria-selected", String(option.dataset.mode === mode));
  }
  query.value = store.preferences.queries[mode] || "";
  query.placeholder = mode === "fs"
    ? "搜索全部索引，例如 *.pdf、soil rain、path:papers Kong" : "搜索最近打开的项目";
  void persist().catch(report);
  schedule(0);
  query.focus();
}

function toggleModes(show?: boolean) {
  const open = show ?? modeMenu.hidden;
  modeMenu.hidden = !open;
  modeToggle.setAttribute("aria-expanded", String(open));
}

async function pin(item: Item) {
  const favorites = store.preferences.favorites;
  store.preferences.favorites = favorites.includes(item.path)
    ? favorites.filter((path) => path !== item.path) : [item.path, ...favorites];
  try {
    await persist();
    schedule(0);
  } catch (error) { report(error); }
}

async function open(item: Item | undefined, action: Action = "default") {
  if (!item || opening) return;
  opening = true;
  menu.hidden = true;
  try {
    // 快捷键勾选“自动复制”时，打开前先复制路径。
    if (autoCopyNext) {
      autoCopyNext = false;
      await invoke("copy_text", { text: item.path }).catch(report);
    }
    await invoke("open_item", { action, mode, path: item.path });
    autoCopyNext = false;
    await invoke("hide_window");
  } catch (error) { report(error); }
  finally { opening = false; }
}

function context(event: MouseEvent, item: Item) {
  event.preventDefault();
  // 右键同时选中该行，操作对象与高亮保持一致。
  selected = items.indexOf(item);
  active();
  menu.replaceChildren();
  const actions: [string, () => void][] = [
    ["用 VSCode 打开", () => void open(item, "vscode")],
    ["用 VSCode Insiders 打开", () => void open(item, "insiders")],
  ];
  if (!item.remote) actions.push(
    ["用 Zed 打开", () => void open(item, "zed")],
    ["打开所在目录", () => void open(item, "directory")],
    ["在终端打开", () => void open(item, "terminal")],
  );
  actions.push(
    [store.preferences.favorites.includes(item.path) ? "取消置顶" : "置顶", () => void pin(item)],
    ["复制路径", () => { void invoke("copy_text", { text: item.path }).catch(report); }],
    ["复制 Linux 路径", () => { void invoke("copy_text", { text: item.path.replaceAll("\\", "/") }).catch(report); }],
  );
  for (const [label, action] of actions) {
    const button = document.createElement("button");
    button.type = "button";
    button.role = "menuitem";
    button.textContent = label;
    button.addEventListener("click", () => { menu.hidden = true; action(); });
    menu.append(button);
  }
  menu.hidden = false;
  menu.querySelector("button")?.focus();
  menu.style.left = `${Math.max(0, Math.min(event.clientX, innerWidth - menu.offsetWidth - 8))}px`;
  menu.style.top = `${Math.max(0, Math.min(event.clientY, innerHeight - menu.offsetHeight - 8))}px`;
}

/* ---------- 快捷键设置 ---------- */

// 组合键文本与 global-hotkey 解析格式一致：macOS 的 ⌘ 记为 Command，其他平台的 Win 键记为 Super。
const IS_MAC = navigator.userAgent.includes("Mac");
const MODIFIER_LABELS: [string, string][] = [
  ["Meta", IS_MAC ? "Command" : "Super"],
  ["Control", "Control"],
  ["Alt", "Alt"],
  ["Shift", "Shift"],
];

function acceleratorOf(event: KeyboardEvent): string | null {
  if (["Meta", "Control", "Alt", "Shift"].includes(event.key)) return null;
  const parts = MODIFIER_LABELS
    .filter(([id]) => event.getModifierState(id))
    .map(([, label]) => label);
  // 使用物理键码，避免空格、Option 字符和非英文输入法导致录制失败。
  const key = event.code.replace(/^(Key|Digit)/, "");
  if (!/^[A-Z0-9]$/.test(key) && !["Space", "Enter", "Tab", "Escape", "Backspace", "Delete",
    "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Home", "End", "PageUp", "PageDown",
    "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12"].includes(key)) {
    return null;
  }
  // 无修饰键的组合容易被系统占用，直接忽略。
  if (!parts.length) return null;
  return [...parts, key].join("+");
}

function shortcutRow(shortcut: Shortcut): HTMLElement {
  const row = document.createElement("div");
  row.className = "row";

  const key = document.createElement("button");
  key.type = "button";
  key.className = "key";
  key.textContent = shortcut.accelerator || "点击录制";
  key.title = "点击后按下组合键";
  key.addEventListener("click", () => {
    recording = { id: shortcut.id, element: key };
    key.classList.add("recording");
    key.textContent = "请按下组合键…";
  });

  const select = document.createElement("select");
  for (const [value, label] of ACTIONS) {
    const option = document.createElement("option");
    option.value = value;
    option.textContent = label;
    option.selected = shortcut.action === value;
    select.append(option);
  }
  select.addEventListener("change", () => {
    shortcut.action = select.value;
    void saveShortcuts();
  });

  const toggle = document.createElement("label");
  toggle.className = "switch";
  toggle.title = "启用/禁用此全局快捷键";
  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  checkbox.setAttribute("aria-label", "启用此全局快捷键");
  checkbox.checked = shortcut.enabled;
  checkbox.addEventListener("change", () => {
    shortcut.enabled = checkbox.checked;
    void saveShortcuts();
  });
  const track = document.createElement("span");
  toggle.append(checkbox, track);

  const remove = document.createElement("button");
  remove.type = "button";
  remove.className = "remove";
  remove.textContent = "🗑";
  remove.title = "删除";
  remove.addEventListener("click", () => {
    store.settings.shortcuts = store.settings.shortcuts.filter((item) => item.id !== shortcut.id);
    renderShortcuts();
    void saveShortcuts();
  });

  row.append(key, select, toggle, remove);
  return row;
}

function renderShortcuts() {
  shortcutRows.replaceChildren(...store.settings.shortcuts.map(shortcutRow));
}

function renderAliases() {
  aliasRows.replaceChildren(...(Object.keys(MODES) as Mode[]).map((key) => {
    const row = document.createElement("div");
    row.className = "row alias";
    const label = document.createElement("span");
    label.className = "alias-name";
    label.textContent = MODES[key];
    const input = document.createElement("input");
    input.type = "text";
    input.placeholder = "留空使用默认名称";
    input.value = store.settings.aliases[key] || "";
    input.addEventListener("change", () => {
      const aliases = { ...store.settings.aliases };
      if (input.value.trim()) aliases[key] = input.value.trim();
      else delete aliases[key];
      store.settings.aliases = aliases;
      modeToggle.textContent = aliases[mode] || MODES[mode];
      void saveSettings();
    });
    row.append(label, input);
    return row;
  }));
}

// 别名随基础设置一起保存。
async function saveSettings() {
  try {
    await invoke("save_settings", { settings: store.settings });
  } catch (error) { report(error); }
}

async function saveShortcuts() {
  try {
    await invoke("save_shortcuts", { shortcuts: store.settings.shortcuts });
  } catch (error) {
    report(error);
    await invoke<Store>("load_store").then((saved) => {
      store.settings.shortcuts = saved.settings.shortcuts;
      renderShortcuts();
    }).catch(report);
  }
}

function settings(visible: boolean) {
  settingsPanel.hidden = !visible;
  panel.hidden = visible;
  menu.hidden = true;
  toggleModes(false);
  clearTimeout(timer);
  ++epoch;
  if (visible) {
    for (const [name, value] of Object.entries(store.settings)) {
      if (name === "shortcuts" || name === "aliases") continue;
      (generalForm.elements.namedItem(name) as HTMLInputElement).value = value;
    }
    renderShortcuts();
    renderAliases();
  } else {
    query.focus();
    schedule(0);
  }
}

query.addEventListener("input", () => { remember(); schedule(); });
modeToggle.addEventListener("click", (event) => { event.stopPropagation(); toggleModes(); });
for (const option of modeMenu.querySelectorAll<HTMLElement>("li[data-mode]")) {
  option.addEventListener("click", () => {
    toggleModes(false);
    setMode(option.dataset.mode as Mode);
  });
}
element("#settings-toggle").addEventListener("click", () => settings(settingsPanel.hidden));
element("#settings-cancel").addEventListener("click", () => settings(false));
element("#hide").addEventListener("click", dismiss);
element("#shortcut-add").addEventListener("click", () => {
  store.settings.shortcuts.push({
    id: `custom-${Date.now()}`,
    accelerator: "",
    action: "fs",
    enabled: true,
    autoCopy: false,
  });
  renderShortcuts();
  void saveShortcuts();
});

for (const tab of document.querySelectorAll<HTMLElement>(".tab[data-tab]")) {
  tab.addEventListener("click", () => {
    for (const other of document.querySelectorAll<HTMLElement>(".tab[data-tab]")) {
      other.setAttribute("aria-selected", String(other === tab));
    }
    for (const pane of document.querySelectorAll<HTMLElement>(".pane[data-pane]")) {
      pane.hidden = pane.dataset.pane !== tab.dataset.tab;
    }
  });
}

generalForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  const updated = {
    ...store.settings,
    ...Object.fromEntries(
      [...new FormData(generalForm)].map(([name, value]) => [name, String(value).trim()]),
    ),
  };
  try {
    await invoke("save_settings", { settings: updated });
    store.settings = updated;
    settings(false);
  } catch (error) { report(error); }
});

document.addEventListener("click", (event) => {
  if (!menu.contains(event.target as Node)) menu.hidden = true;
  if (!modeMenu.contains(event.target as Node)) toggleModes(false);
});
document.addEventListener("keydown", (event) => {
  if (recording) {
    // 录制期间独占键盘输入，Esc 取消。
    event.preventDefault();
    event.stopPropagation();
    if (event.key === "Escape") {
      recording = null;
      renderShortcuts();
      return;
    }
    const accelerator = acceleratorOf(event);
    if (!accelerator) return;
    const shortcut = store.settings.shortcuts.find((item) => item.id === recording!.id);
    recording = null;
    if (shortcut) {
      shortcut.accelerator = accelerator;
      renderShortcuts();
      void saveShortcuts();
    }
    return;
  }
  if (event.key === "Escape") {
    event.preventDefault();
    if (!menu.hidden) { menu.hidden = true; query.focus(); }
    else if (!modeMenu.hidden) { toggleModes(false); query.focus(); }
    else if (settingsPanel.hidden) dismiss();
    else settings(false);
    return;
  }
  if (!menu.hidden) {
    // 右键菜单内用上下键切换、回车执行。
    const buttons = [...menu.querySelectorAll("button")];
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const step = event.key === "ArrowDown" ? 1 : -1;
      const current = buttons.indexOf(document.activeElement as HTMLButtonElement);
      buttons[(current + step + buttons.length) % buttons.length]?.focus();
    }
    return;
  }
  if (!settingsPanel.hidden || !store) return;
  if ((event.ctrlKey || event.metaKey) && /^[1-9]$/.test(event.key)) {
    // 与 uTools 一致：⌘/Ctrl + 数字直接打开对应结果。
    event.preventDefault();
    const item = items[Number(event.key) - 1];
    if (item) void open(item);
  } else if (["ArrowDown", "ArrowUp", "PageDown", "PageUp"].includes(event.key)) {
    event.preventDefault();
    const direction = event.key.endsWith("Down") ? 1 : -1;
    const step = event.key.startsWith("Page") ? 10 : 1;
    selected = Math.max(0, Math.min(items.length - 1, selected + direction * step));
    active(true);
  } else if (event.key === "Enter" && items[selected]) {
    event.preventDefault();
    if (event.ctrlKey || event.metaKey) void pin(items[selected]);
    else void open(items[selected]);
  }
});

async function init() {
  store = await invoke<Store>("load_store");
  await listen<{ mode: string; autoCopy: boolean }>("shortcut-triggered", (event) => {
    const next = event.payload.mode as Mode;
    if (!settingsPanel.hidden) settings(false);
    autoCopyNext = event.payload.autoCopy;
    if (next in MODES) setMode(next);
    query.focus();
    query.select();
  });
  await listen("launcher-show", () => {
    if (settingsPanel.hidden) {
      query.focus();
      query.select();
      schedule(0);
    }
  });
  const saved = store.preferences.mode;
  setMode(saved === "fs" || saved === "zed" ? saved : "vscode");
}

void init().catch(report);

import { exec, execFile } from 'child_process';
import { statSync } from 'fs';
import { dirname } from 'path';
import { Setting } from 'utools-helper';
import { check_path, editorCmd, quoteArg } from './plugin';

const MENU_ACTIONS = {
    TERMINAL: 'terminal',
    EXPLORER: 'explorer',
    VSCODE: 'vscode',
    ZED: 'zed',
    COPY_LINUX_PATH: 'copy_path_linux'
} as const;

function openTerminal(dir: string) {
    const terminal = (Setting.Get("terminal") || "").trim();
    // 新实例确保 kitty 已运行时仍能接收工作目录参数。
    if (process.platform === "darwin" && !terminal) {
        execFile("/usr/bin/open", ["-na", "kitty", "--args", "--directory", dir], reportError);
        return;
    }
    const bin = terminal || (process.platform === "linux" ? "gnome-terminal --working-directory" : "wt.exe -d");
    exec(`${bin} ${quoteArg(dir)}`, reportError);
}

function handleContextAction(action: string, path: string) {
    if (action === MENU_ACTIONS.COPY_LINUX_PATH) {
        navigator.clipboard.writeText(path.replace(/\\/g, "/"));
        return;
    }

    const isRemote = /vscode-remote:/i.test(path);

    if (action === MENU_ACTIONS.TERMINAL || action === MENU_ACTIONS.EXPLORER) {
        try {
            if (statSync(path).isFile()) path = dirname(path);
        } catch {
            console.error("Path does not exist:", path);
            return;
        }
        if (process.platform === "win32" && !isRemote) path = path.replace(/\//g, "\\");
        if (action === MENU_ACTIONS.TERMINAL) return openTerminal(path);

        const explorer = (Setting.Get("explorer") || "").trim()
            || (process.platform === "darwin" ? "/usr/bin/open" : process.platform === "linux" ? "xdg-open" : "explorer.exe");
        const bin = process.platform === "darwin" && explorer === "open" ? "/usr/bin/open" : explorer;
        exec(`${bin} ${quoteArg(path)}`, reportError);
        return;
    }

    const cmd = action === MENU_ACTIONS.VSCODE
        ? editorCmd(Setting.Get("code") || "code", path, Setting.Get("shell") || "")
        : editorCmd("zed", path, Setting.Get("shell") || "");
    console.log(cmd);
    exec(cmd, reportError);
}

function reportError(error: Error | null) {
    if (error) utools.showNotification(`打开失败：${error.message}`);
}

function hideMenu(menu: HTMLElement) {
    menu.style.display = 'none';
}

function showMenu(menu: HTMLElement, x: number, y: number) {
    menu.style.display = 'block';
    menu.style.left = `${x}px`;
    menu.style.top = `${y}px`;
}

function setupContextMenu() {
    const menu = document.createElement('div');
    menu.className = 'context-menu';
    menu.innerHTML = `
    <div data-action="${MENU_ACTIONS.TERMINAL}">Open in Terminal</div>
    <div data-action="${MENU_ACTIONS.EXPLORER}">Open in Directory</div>
    <div data-action="${MENU_ACTIONS.VSCODE}">Open in VSCode</div>
    <div data-action="${MENU_ACTIONS.ZED}">Open in Zed</div>
    <div data-action="${MENU_ACTIONS.COPY_LINUX_PATH}">Copy Linux Path</div>
  `;
    document.body.appendChild(menu);

    // 点击空白处隐藏菜单
    document.addEventListener('click', () => hideMenu(menu));

    // 鼠标移出菜单区域时隐藏
    document.addEventListener('mousemove', (e) => {
        if (menu.style.display === 'block') {
            const rect = menu.getBoundingClientRect();
            const isOutside = e.clientX < rect.left || e.clientX > rect.right ||
                e.clientY < rect.top || e.clientY > rect.bottom;
            if (isOutside) hideMenu(menu);
        }
    });
}

function onContextmenu(e: MouseEvent) {
    const target = e.target as HTMLElement;
    const descElement = target.closest(".list-item")?.querySelector(".list-item-description") as HTMLElement;

    if (!descElement) return;

    e.preventDefault();
    const menu = document.querySelector('.context-menu') as HTMLElement;
    if (!menu) return;

    // 获取并处理路径
    const path = check_path(descElement.innerText.trim());
    const isRemote = /vscode-remote:/i.test(path);

    showMenu(menu, e.pageX, e.pageY);
    console.log('path:', path, 'isRemote:', isRemote);

    // 配置菜单项显示状态和点击事件
    menu.querySelectorAll('div').forEach(menuItem => {
        const action = menuItem.getAttribute('data-action');
        const htmlItem = menuItem as HTMLElement;

        // Remote 项目只显示 VSCode 相关操作
        if (isRemote) {
            const allowedActions = [MENU_ACTIONS.VSCODE, MENU_ACTIONS.COPY_LINUX_PATH];
            htmlItem.style.display = allowedActions.includes(action as any) ? 'block' : 'none';
        } else {
            htmlItem.style.display = 'block';
        }

        htmlItem.onclick = () => {
            handleContextAction(action!, path);
            hideMenu(menu);
        };
    });
}

function InitContextMenu() {
    setupContextMenu();

    const style = document.createElement('style');
    style.innerHTML = `
    .context-menu {
        position: absolute;
        background: white;
        border: 1px solid #ccc;
        box-shadow: 2px 2px 5px rgba(0, 0, 0, 0.2);
        z-index: 1000;
        display: none;
    }
    .context-menu div {
        padding: 5px 10px;
        cursor: pointer;
    }
    .context-menu div:hover {
        background: #f0f0f0;
    }
    `;
    document.head.appendChild(style);
}

export { InitContextMenu, onContextmenu };

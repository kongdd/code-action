import { exec } from 'child_process';
import { statSync } from 'fs';
import { dirname } from 'path';
import { Setting } from 'utools-helper';

const MENU_ACTIONS = {
    TERMINAL: 'terminal',
    EXPLORER: 'explorer',
    VSCODE: 'vscode',
    VSCODE_INSIDER: 'vscode-insider',
    COPY_LINUX_PATH: 'copy_path_linux'
} as const;

function handleContextAction(action: string, path: string) {
    // 复制 Linux 路径
    if (action === MENU_ACTIONS.COPY_LINUX_PATH) {
        navigator.clipboard.writeText(path.replace(/\\/g, '/'));
        return;
    }

    const isRemote = path.includes('vscode-remote') || path.includes('remote');

    // 对于终端和文件管理器，如果是文件则使用父目录
    if ([MENU_ACTIONS.TERMINAL, MENU_ACTIONS.EXPLORER].includes(action as any)) {
        try {
            if (statSync(path).isFile()) {
                path = dirname(path);
            }
        } catch (error) {
            console.error('Path does not exist:', path);
            return;
        }

        // Windows 路径转换（仅本地路径）
        if (process.platform === 'win32' && !isRemote) {
            path = path.replace(/\//g, '\\');
        }
    }

    // 构建命令
    let cmd = '';
    switch (action) {
        case MENU_ACTIONS.TERMINAL:
            cmd = `${Setting.Get("terminal") || "wt.exe -d"} "${path}"`;
            break;
        case MENU_ACTIONS.EXPLORER:
            cmd = `${Setting.Get("explorer") || "explorer.exe"} "${path}"`;
            break;
        case MENU_ACTIONS.VSCODE:
        case MENU_ACTIONS.VSCODE_INSIDER:
            const vsCmd = action === MENU_ACTIONS.VSCODE ? (Setting.Get("code") || "code") : "code-insiders";
            const uriFlag = path.includes(".code-workspace") ? "--file-uri" : "--folder-uri";
            cmd = `${vsCmd} ${uriFlag} "${path}"`;
            break;
        default:
            cmd = `${action} "${path}"`;
    }

    console.log(cmd);
    exec(cmd);
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
    <div data-action="${MENU_ACTIONS.VSCODE_INSIDER}">Open in VSCode Insider</div>
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
    const descElement = target.parentElement?.querySelector(".list-item-description") as HTMLElement;

    if (!descElement) return;

    e.preventDefault();
    const menu = document.querySelector('.context-menu') as HTMLElement;
    if (!menu) return;

    // 获取并处理路径
    let path = descElement.innerText.replace(/^file:\/\/\//, '');
    const isRemote = path.includes('vscode-remote') || path.includes('remote');

    showMenu(menu, e.pageX, e.pageY);
    console.log('path:', path, 'isRemote:', isRemote);

    // 配置菜单项显示状态和点击事件
    menu.querySelectorAll('div').forEach(menuItem => {
        const action = menuItem.getAttribute('data-action');
        const htmlItem = menuItem as HTMLElement;

        // Remote 项目只显示 VSCode 相关操作
        if (isRemote) {
            const allowedActions = [MENU_ACTIONS.VSCODE, MENU_ACTIONS.VSCODE_INSIDER, MENU_ACTIONS.COPY_LINUX_PATH];
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

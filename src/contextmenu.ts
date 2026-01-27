import { exec } from 'child_process';
import { statSync } from 'fs';
import { dirname } from 'path';
import { Setting } from 'utools-helper';
// import { platform } from 'os';
// import clipboard from 'clipboardy';

function handleContextAction(action: string, path: string) {
    if (action == "copy_path_linux") {
        path = path.replace('\\', '/');
        navigator.clipboard.writeText(path);
        return;
    }

    // 获取配置的命令
    const terminalCmd = Setting.Get("terminal") || "wt.exe -d";
    const explorerCmd = Setting.Get("explorer") || "explorer.exe";
    const codeCmd = Setting.Get("code") || "code";

    // 对于终端和文件管理器,如果是文件则使用父目录
    if (action == "terminal" || action == "explorer") {
        try {
            if (statSync(path).isFile()) {
                path = dirname(path);
            }
        } catch (error) {
            console.error('Path does not exist:', path);
            return;
        }
    }

    // Windows 路径转换 (仅对本地路径)
    let dirActions = ["terminal", "explorer"];
    const isRemote = path.includes('vscode-remote') || path.includes('remote');
    if (dirActions.includes(action) && process.platform === 'win32' && !isRemote) {
        path = path.replace(/\//g, '\\');
    }

    // 根据 action 类型选择命令
    let cmd = "";
    switch (action) {
        case "terminal":
            cmd = `${terminalCmd} "${path}"`;
            break;
        case "explorer":
            cmd = `${explorerCmd} "${path}"`;
            break;
        case "vscode":
        case "vscode-insider":
            // VSCode 需要使用 --folder-uri 或 --file-uri 参数
            const vsCmd = action === "vscode" ? codeCmd : "code-insiders";
            if (path.includes(".code-workspace")) {
                cmd = `${vsCmd} --file-uri "${path}"`;
            } else {
                cmd = `${vsCmd} --folder-uri "${path}"`;
            }
            break;
        default:
            cmd = `${action} "${path}"`;
    }

    console.log(cmd);

    new Promise((resolve, reject) => {
        exec(cmd);
    });
}

function setupContextMenu() {
    // const menus = document.querySelectorAll('.context-menu');
    // if (menus) return; // Already setup
    const menu = document.createElement('div');
    menu.className = 'context-menu';
    menu.innerHTML = `
        <div data-action="terminal">Open in Terminal</div>
        <div data-action="explorer">Open in Directory</div>
        <div data-action="vscode">Open in VSCode</div>
        <div data-action="vscode-insider">Open in VSCode Insider</div>
        <div data-action="copy_path_linux">Copy Linux Path</div>
      `;
    document.body.appendChild(menu);
    document.addEventListener('click', () => menu.style.display = 'none');
    // document.addEventListener('mousemove', () => menu.style.display = 'none');
    // Hide menu when mouse moves outside its bounds
    document.addEventListener('mousemove', (e) => {
        if (menu.style.display === 'block') {
            const rect = menu.getBoundingClientRect();
            const isOutside =
                e.clientX < rect.left ||
                e.clientX > rect.right ||
                e.clientY < rect.top ||
                e.clientY > rect.bottom;

            if (isOutside) {
                menu.style.display = 'none';
            }
        }
    });

    const items = document.querySelectorAll('.list-item');
    items.forEach(item => {
        item.addEventListener('contextmenu', (e: any) => {
            e.preventDefault();
            const path = item.getAttribute('data-path');
            menu.style.display = 'block';
            menu.style.left = `${e.pageX}px`;
            menu.style.top = `${e.pageY}px`;

            menu.querySelectorAll('div').forEach(menuItem => {
                menuItem.onclick = () => {
                    const action = menuItem.getAttribute('data-action');
                    handleContextAction(action, path);
                    menu.style.display = 'none';
                };
            });
        });
    });
}


function insertCSS(css: string) {
    // Create a new style element
    const style = document.createElement('style');
    // Append to document head
    style.innerHTML = css;
    document.head.appendChild(style);
};


function onContextmenu(e: any) {
    let div = e.target.parentElement.querySelector(".list-item-description");;
    console.log(div);

    if (div) {
        e.preventDefault();
        var menu = document.querySelector('.context-menu') as HTMLElement;;
        var path = div.innerText;

        // 处理路径格式
        if (path.startsWith('file:///')) {
            path = path.replace('file:///', '');
        }

        const isRemote = path.includes('vscode-remote') || path.includes('remote');

        // const path = item.getAttribute('data-path');
        menu.style.display = 'block';
        menu.style.left = `${e.pageX}px`;
        menu.style.top = `${e.pageY}px`;
        console.log('path:', path, 'isRemote:', isRemote);

        menu.querySelectorAll('div').forEach(menuItem => {
            const action = menuItem.getAttribute('data-action');

            // remote 项目只显示特定菜单项
            if (isRemote) {
                // 只允许 vscode 和 copy_path_linux 操作
                if (action === 'vscode' || action === 'vscode-insider' || action === 'copy_path_linux') {
                    (menuItem as HTMLElement).style.display = 'block';
                } else {
                    (menuItem as HTMLElement).style.display = 'none';
                }
            } else {
                // 本地项目显示所有菜单项
                (menuItem as HTMLElement).style.display = 'block';
            }

            menuItem.onclick = () => {
                handleContextAction(action, path);
                menu.style.display = 'none';
            };
        });
        console.log('oncontextmenu clicked:', e.target.textContent);
        // 在这里添加你的点击处理逻辑
    }
    // setupContextMenu();
    // console.log("右键菜单被触发");
};

function InitContextMenu() {
    setupContextMenu();
    insertCSS(`
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
  `)
}

export {
    handleContextAction,
    setupContextMenu,
    insertCSS,
    onContextmenu,
    InitContextMenu,
    exec
}

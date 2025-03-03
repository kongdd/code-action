import { exec } from 'child_process';
import { statSync } from 'fs';
import { dirname } from 'path';
// import { platform } from 'os';
// import clipboard from 'clipboardy';

function handleContextAction(action: string, path: string) {
  if (action == "copy_path_linux") {
    path = path.replace('\\', '\/');
    navigator.clipboard.writeText(path);
    return;
  }

  if (action == "explorer.exe" && statSync(path).isFile()) {
    path = dirname(path);
  }
  if (process.platform === 'win32') {
    path = path.replace(/\//g, '\\');
  }
  var cmd = `${action} "${path}"`;
  console.log(cmd);

  // switch (action) {
  //   case 'vscode':
  //     // alert(`Would open ${path} in VSCode`);
  //     break;
  //   case 'directory':
  //     // alert(`Would open directory of ${path}`);
  //     break;
  //   case 'terminal':
  //     // alert(`Would open terminal at ${path}`);
  //     break;
  //   case 'vscode-insider':
  //     // alert(`Would open ${path} in VSCode Insider`);
  //     break;
  // }
  new Promise((resolve, reject) => {
    exec(cmd);
  });
  // execCmd(cmd, {
  //   timeout: 100, windowsHide: true,
  //   encoding: "utf-8"
  // })
  //   // .then(() => utools.hideMainWindow())
  //   .catch((reason: any) => {
  //     console.log(reason);
  //   });
}

function setupContextMenu() {
  // const menus = document.querySelectorAll('.context-menu');
  // if (menus) return; // Already setup
  const menu = document.createElement('div');
  menu.className = 'context-menu';
  menu.innerHTML = `
        <div data-action="wt.exe -d">Open in Terminal</div>
        <div data-action="explorer.exe">Open in Directory</div>
        <div data-action="code.exe">Open in VSCode</div>
        <div data-action="code-insiders.cmd">Open in VSCode Insider</div>
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
    var path = div.innerText.replace('file:///', '');

    // const path = item.getAttribute('data-path');
    menu.style.display = 'block';
    menu.style.left = `${e.pageX}px`;
    menu.style.top = `${e.pageY}px`;
    console.log('path:', path);

    menu.querySelectorAll('div').forEach(menuItem => {
      menuItem.onclick = () => {
        const action = menuItem.getAttribute('data-action');
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

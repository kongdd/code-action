import { exec } from "child_process";

var path = "e:/github/kongdd/utools"

if (process.platform === 'win32') {
  path = path.replace(/\//g, "\\")
}

// var app = "explorer.exe";
// var app = "code-insiders.cmd";
var app = "wt.exe -d"

var cmd = `${app} "${path}"`
exec(cmd)
console.log(cmd)

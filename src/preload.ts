import { InitPlugins, Setting } from "utools-helper";
import { config } from "./config";
import { VSCode } from "./vscode";
import { Everything } from "./everything";

try {
  InitPlugins([
    new VSCode(), 
    new Everything(),
    Setting.Init("vsc-setting", config)
  ]);
} catch (error) {
  alert(error.stack ? error.stack : error);
}

import { InitPlugins, Setting } from "utools-helper";
import { config } from "./config";
import { VSCode } from "./vscode";
import { Zed } from "./zed";

try {
  InitPlugins([
    new VSCode(),
    new Zed(),
    Setting.Init("vsc-setting", config)
  ]);
} catch (error) {
  alert(error.stack ? error.stack : error);
}

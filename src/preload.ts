import { InitPlugins, Setting } from "utools-helper";
import { config } from "./config";
import { VSCode } from "./vscode";
import { Zed } from "./zed";
import { FileSearch } from "./fs";

try {
  InitPlugins([
    new VSCode(),
    new Zed(),
    new FileSearch(),
    Setting.Init("vsc-setting", config)
  ]);
} catch (error) {
  alert(error.stack ? error.stack : error);
}

import { readdirSync } from "fs";
import { join } from "path";
import * as util from "util";
import { exec } from "child_process";

export function getIcon(ext: string): string {
  let icons = readdirSync(join(__dirname, "..", "icon"));
  let icon = icons.find((icon) => {
    return "." + icon.split(".")[0] === ext.toLowerCase();
  });
  if (!icon && !ext) icon = "folder.svg";
  if (!icon && ext) icon = "file.svg";
  return join("icon", icon);
}

export function check_path(path: string): string {
  // console.log(process.platform, path);
  if (process.platform == "win32")  {
    // path = decodeURIComponent(path);
    return path.replace(/\\/g, "/").replace("file:///", "");
  }
  return path;
}

// Promisify the exec function
export const execPromise = util.promisify(exec);

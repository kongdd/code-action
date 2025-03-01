import { readdirSync } from "fs";
import { join } from "path";


export function getIcon(ext: string): string {
  let icons = readdirSync(join(__dirname, "..", "icon"));
  let icon = icons.find((icon) => {
    return "." + icon.split(".")[0] === ext.toLowerCase();
  });
  if (!icon && !ext) icon = "folder.svg";
  if (!icon && ext) icon = "file.svg";
  return join("icon", icon);
}

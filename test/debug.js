import { exec } from "child_process";
import path from "path";
import * as util from "util";

// Promisify the exec function
const execPromise = util.promisify(exec);

async function runCommand(cmd, maxBuffer) {
  try {
    const { stdout, stderr } = await execPromise(cmd, { maxBuffer: maxBuffer });
    // console.log(`stdout: ${stdout}`);
    if (stderr) console.error(`stderr: ${stderr}`);
    const filePaths = stdout.split('\r\n').filter(line => line.trim() !== '');
    return filePaths;
  } catch (error) {
    console.error(`Execution error: ${error}`);
    throw error; // Or handle it as needed
  }
}

async function GetFile(pattern) {
  pattern = pattern || "Liu *.pdf";
  const app = path.resolve("../bin/es.exe");
  // const pattern = "soil moisture";

  const cmd = `"${app}" /o-d -n 10 "${pattern}"`; // e.g., "C:\path\to\es.exe" "soil moisture"
  const maxBuffer = 1024 * 1024; // 1MB buffer
  const result = await runCommand(cmd, maxBuffer);

  console.log(result);  
  return result;
}

// Define the command
// Execute the command
(async () => {
  GetFile()
})();

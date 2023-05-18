import fs from "fs";
import path from "path";

const distPath = "dist";
const serverPath = path.join("..", "backend", "public");

// Remove the old public folder if it exists
if (fs.existsSync(serverPath)) {
  fs.rmSync(serverPath, {
    recursive: true,
    force: true
  });
  console.log("Removed old files");
}

if (fs.existsSync(distPath)) {
  fs.renameSync(distPath, serverPath);
  console.log("Moved dist");
} else {
  console.error("Client dist missing");
}

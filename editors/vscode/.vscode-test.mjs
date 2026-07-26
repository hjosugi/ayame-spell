import { defineConfig } from "@vscode/test-cli";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

const here = dirname(fileURLToPath(import.meta.url));
export default defineConfig({
  files: "out/test/**/*.test.js",
  version: "stable",
  workspaceFolder: join(here, "test-fixtures"),
  launchArgs: ["--disable-gpu", "--disable-dev-shm-usage", "--headless"],
  mocha: {
    timeout: 30000,
  },
});

import esbuild from "esbuild";
import { rm } from "node:fs/promises";

const production = process.argv.includes("--production");

if (production) {
  await rm("dist/extension.js.map", { force: true });
}

await esbuild.build({
  entryPoints: ["src/extension.ts"],
  bundle: true,
  outfile: "dist/extension.js",
  external: ["vscode"],
  format: "cjs",
  platform: "node",
  target: "node18",
  sourcemap: !production,
  minify: production,
});

if (!production) {
  await esbuild.build({
    entryPoints: ["test/suite/*.test.ts"],
    bundle: true,
    outdir: "out/test",
    external: ["vscode"],
    format: "cjs",
    platform: "node",
    target: "node18",
    sourcemap: true,
  });
}

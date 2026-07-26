import * as path from "path";

export type SupportedTarget =
  "darwin-arm64" | "darwin-x64" | "linux-arm64" | "linux-x64" | "win32-x64";

const TARGETS: Record<string, SupportedTarget> = {
  "darwin-arm64": "darwin-arm64",
  "darwin-x64": "darwin-x64",
  "linux-arm64": "linux-arm64",
  "linux-x64": "linux-x64",
  "win32-x64": "win32-x64",
};

export function supportedTarget(
  platform: NodeJS.Platform = process.platform,
  arch: string = process.arch,
): SupportedTarget | undefined {
  return TARGETS[`${platform}-${arch}`];
}

export function executableName(
  platform: NodeJS.Platform = process.platform,
): string {
  return platform === "win32" ? "ayame-spell.exe" : "ayame-spell";
}

export function bundledServerPath(
  extensionPath: string,
  platform: NodeJS.Platform = process.platform,
  arch: string = process.arch,
): string | undefined {
  const target = supportedTarget(platform, arch);
  return target
    ? path.join(extensionPath, "server", target, executableName(platform))
    : undefined;
}

export function versionsCompatible(
  extensionVersion: string,
  serverVersion: string,
): boolean {
  const pair = (version: string): string =>
    version
      .replace(/^v/, "")
      .split(/[+-]/, 1)[0]
      .split(".")
      .slice(0, 2)
      .join(".");
  return pair(extensionVersion) === pair(serverVersion);
}

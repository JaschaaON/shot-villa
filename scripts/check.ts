/** Führt die Projektprüfungen für die unterstützten Zielplattformen aus. */
import { execFileSync } from "node:child_process";
import path from "node:path";
import process from "node:process";

const root = process.cwd();
const rustManifest = path.join(root, "src-tauri", "Cargo.toml");
const cargo = process.platform === "win32" ? "cargo.exe" : "cargo";
const packageManager = process.platform === "win32" ? "pnpm.cmd" : "pnpm";

const run = (
  command: string,
  args: string[],
  env: NodeJS.ProcessEnv = process.env
): void => {
  execFileSync(command, args, { cwd: root, env, stdio: "inherit" });
};

const cargoBin = path.join(process.env.HOME ?? "", ".cargo", "bin");
const baseEnv: NodeJS.ProcessEnv = {
  ...process.env,
  PATH: [cargoBin, process.env.PATH ?? ""].join(path.delimiter),
};

const runCargo = (args: string[], env = baseEnv): void => {
  const separator = args.indexOf("--");
  const commandArgs = [...args];
  commandArgs.splice(
    separator === -1 ? commandArgs.length : separator,
    0,
    "--manifest-path",
    rustManifest
  );
  run(cargo, commandArgs, env);
};

const crossEnv = (): NodeJS.ProcessEnv => {
  const zigcc = path.join(process.env.HOME ?? "", ".local", "opt", "zigcc");
  const crossCargoBin = path.join(process.env.HOME ?? "", ".cargo", "bin");
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    AR: path.join(zigcc, "ar"),
    AR_aarch64_apple_darwin: path.join(zigcc, "stubar"),
    AR_x86_64_pc_windows_msvc: path.join(zigcc, "stubar"),
    CARGO_TARGET_DIR: path.join(
      process.env.TMPDIR ?? "/tmp",
      "shot-villa-cargo-target"
    ),
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER: path.join(zigcc, "cc"),
    CC: path.join(zigcc, "cc"),
    CC_aarch64_apple_darwin: path.join(zigcc, "stubcc"),
    CC_x86_64_pc_windows_msvc: path.join(zigcc, "stubcc"),
    PATH: [crossCargoBin, zigcc, process.env.PATH ?? ""].join(path.delimiter),
  };
  return env;
};

const clippy = (target?: string, env?: NodeJS.ProcessEnv): void => {
  const args = ["clippy", "--locked"];
  if (target !== undefined) {
    args.push("--target", target);
  }
  args.push("--all-targets", "--", "-D", "warnings");
  runCargo(args, env);
};

run(packageManager, ["lint"]);
run(packageManager, ["typecheck"]);
run(cargo, ["fmt", "--manifest-path", rustManifest, "--check"], baseEnv);

if (process.platform === "darwin") {
  clippy();
  clippy("x86_64-pc-windows-msvc");
} else if (process.platform === "linux") {
  const env = crossEnv();
  clippy("aarch64-apple-darwin", env);
  clippy("x86_64-pc-windows-msvc", env);
} else if (process.platform === "win32") {
  clippy();
} else {
  throw new Error(`Nicht unterstützte Prüfplattform: ${process.platform}`);
}

run(packageManager, ["test"]);
if (process.platform === "linux") {
  process.stdout.write(
    "Rust-Tests übersprungen: Die App unterstützt Linux nicht; Rust-Tests laufen auf macOS und Windows.\n"
  );
} else {
  runCargo(["test"]);
}

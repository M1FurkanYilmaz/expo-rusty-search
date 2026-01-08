import path from "path";
import fs from "fs";
import { spawnSync } from "child_process";

const TARGETS = {
  ios: "aarch64-apple-ios",
  "ios-sim": "aarch64-apple-ios-sim",
};

// The name of your Rust folder
const RUST_CRATE_DIR = "expo-search-core";
// The expected name of the output library (Rust converts dashes to underscores)
const LIB_NAME = "libexpo_search_core.a";
const HEADER_NAME = "expo_search_core.h"; // This name is defined in cbindgen or manually

function cargoBuild(target: string) {
  console.log(`Compiling for ${target}...`);
  const result = spawnSync(
    "cargo",
    ["build", "--release", "--target", target],
    {
      stdio: "inherit",
    }
  );

  if (result.status !== 0) {
    console.error(`Failed to compile for ${target}`);
    process.exit(1);
  }
}

function getTarget() {
  const args = process.argv.slice(2);
  // Default to ios-sim if no argument provided, for easier dev experience
  const targetKey = (args[0] ?? "").replace("--target=", "");

  if (targetKey === "ios" || targetKey === "ios-sim") {
    return targetKey;
  }

  // You can change this default to 'ios' (device) if you prefer
  console.warn("⚠️ No target specified. Defaulting to 'ios-sim' (Simulator).");
  console.warn("   Usage: npm run cargo-ios -- --target=ios (for device)");
  return "ios-sim";
}

function main() {
  const targetKey = getTarget(); // "ios" or "ios-sim"
  const rustTarget = TARGETS[targetKey as keyof typeof TARGETS];

  console.log(`🚀 Starting iOS Build for: ${targetKey} (${rustTarget})`);

  const rootDir = process.cwd();
  const rustDir = path.join(rootDir, RUST_CRATE_DIR);

  // 1. Enter Rust directory
  process.chdir(rustDir);

  // 2. Build the Rust Library
  cargoBuild(rustTarget);

  // 3. Generate C Headers using cbindgen
  console.log("📝 Generating C headers...");
  spawnSync(
    "cbindgen",
    [
      "--lang",
      "c",
      "--crate",
      "expo-search-core", // Matches name in Cargo.toml
      "--output",
      HEADER_NAME,
    ],
    { stdio: "inherit" }
  );

  // 4. Return to root
  process.chdir(rootDir);

  // 5. Define Paths for Copying
  const destDir = path.join(rootDir, "ios", "rust");

  const srcLibPath = path.join(
    rustDir,
    "target",
    rustTarget,
    "release",
    LIB_NAME
  );

  const srcHeaderPath = path.join(rustDir, HEADER_NAME);

  // 6. Ensure destination exists
  if (!fs.existsSync(destDir)) {
    fs.mkdirSync(destDir, { recursive: true });
  }

  // 7. Copy Files
  if (fs.existsSync(srcLibPath)) {
    fs.copyFileSync(srcLibPath, path.join(destDir, LIB_NAME));
    console.log(`✅ Copied library to ios/rust/${LIB_NAME}`);
  } else {
    console.error(`❌ Library not found at ${srcLibPath}`);
    process.exit(1);
  }

  if (fs.existsSync(srcHeaderPath)) {
    fs.copyFileSync(srcHeaderPath, path.join(destDir, HEADER_NAME));
    console.log(`✅ Copied headers to ios/rust/${HEADER_NAME}`);
  } else {
    console.error(`❌ Headers not found at ${srcHeaderPath}`);
  }

  console.log("✨ iOS build complete!");
}

main();

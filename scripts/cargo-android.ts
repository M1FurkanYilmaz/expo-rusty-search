import path from "path";
import fs from "fs";
import { spawnSync } from "child_process";

// Map Rust targets to Android ABI folder names
const TARGET_TO_DESTINATION = {
  "aarch64-linux-android": "arm64-v8a",
  "armv7-linux-androideabi": "armeabi-v7a",
  "i686-linux-android": "x86",
  "x86_64-linux-android": "x86_64",
} as const;

// The name of your Rust folder
const RUST_CRATE_DIR = "expo-search-core";
// The expected name of the output binary (Rust converts dashes to underscores)
const LIB_NAME = "libexpo_search_core.so";

function build(target: string) {
  console.log(`Building for target: ${target}`);
  const result = spawnSync(
    "cargo",
    ["ndk", "--target", target, "--platform", "31", "build", "--release"],
    {
      stdio: "inherit",
      env: { ...process.env },
    }
  );

  if (result.status !== 0) {
    console.error(`Failed to build for ${target}`);
    process.exit(1);
  }
}

function main() {
  console.log("🚀 Starting Android Build Process...");
  const rootDir = process.cwd();
  const rustDir = path.join(rootDir, RUST_CRATE_DIR);

  // 1. Enter Rust directory
  process.chdir(rustDir);

  // 2. Build for all targets
  Object.keys(TARGET_TO_DESTINATION).forEach(build);

  // 3. Go back to root to handle copying
  process.chdir(rootDir);

  console.log("📂 Copying binaries to android/src/main/jniLibs...");

  Object.entries(TARGET_TO_DESTINATION).forEach(([target, architecture]) => {
    const sourcePath = path.join(
      rustDir,
      "target",
      target,
      "release",
      LIB_NAME
    );

    const architectureDestDir = path.join(
      rootDir,
      "android",
      "src",
      "main",
      "jniLibs",
      architecture
    );

    // Create destination folder if it doesn't exist
    if (!fs.existsSync(architectureDestDir)) {
      fs.mkdirSync(architectureDestDir, { recursive: true });
    }

    const destPath = path.join(architectureDestDir, LIB_NAME);

    if (fs.existsSync(sourcePath)) {
      fs.copyFileSync(sourcePath, destPath);
      console.log(`✅ Copied ${architecture}`);
    } else {
      console.error(`❌ Could not find binary at ${sourcePath}`);
    }
  });

  console.log("✨ Android build complete!");
}

main();

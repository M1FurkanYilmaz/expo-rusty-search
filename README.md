# expo-rusty-search

A high-performance, full-text search engine for Expo (React Native) powered by [Tantivy](https://github.com/quickwit-oss/tantivy) and Rust.

## 🚀 Why Rust for Search?

Standard JavaScript-based search libraries (like FlexSearch or Lunr) often struggle with large datasets, leading to UI thread blocking and high memory consumption on mobile devices.

`expo-rusty-search` offloads heavy indexing and querying to **Tantivy**, a search engine library written in Rust which is often faster than Lucene. By using the JNI (Android) and C-interop (iOS), we achieve near-native performance for complex queries.

---

## 🛠 Architecture

This module follows the **Expo Modules API** structure:

- **`expo-search-core/`**: The Rust library. Contains the logic for Tantivy indices, document management, and bulk indexing.
- **`ios/`**: Swift wrapper that communicates with the Rust static library via C headers.
- **`android/`**: Kotlin wrapper that communicates with the Rust shared objects (`.so`) via JNI.
- **`src/`**: TypeScript interface for your React Native application.

---

## 📦 Installation

```bash
npm install expo-rusty-search
```

## 💻 Usage

```TypeScript

import {
initializeIndex,
addDocumentsBulk,
searchDocuments
} from 'expo-rusty-search';

// 1. Initialize the in-memory index
await initializeIndex();

// 2. Add documents in bulk (JSON string)
const docs = [
{ title: "Rust", body: "A language empowering everyone to build reliable and efficient software." },
{ title: "Expo", body: "A framework for universal React applications." }
];
await addDocumentsBulk(JSON.stringify(docs));

// 3. Search
const results = await searchDocuments("reliable software");
console.log(JSON.parse(results));
```

# 🏗 Development & Building

Since this module contains native Rust code, you must compile the Rust binaries before building the app.
Prerequisites

    Rust and Cargo

    cargo-ndk (for Android): cargo install cargo-ndk

    cbindgen (for iOS headers): cargo install cbindgen

    Rust Targets:
    Bash

    rustup target add aarch64-apple-ios aarch64-apple-ios-sim
    rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

# Building the Native Libraries

We provide automated scripts to compile Rust and move the binaries to the correct folders:
Command Platform Description
npm run build:android Android Builds .so files for all 4 ABIs and moves them to jniLibs.
npm run build:ios iOS (Sim) Builds .a for Apple Silicon Simulator and generates C headers.
npm run build:ios:device iOS (Device) Builds .a for physical iPhone (ARM64).
🤝 Contributing

We welcome contributions! Please follow these steps to get started:

    Fork the repo and create your branch.

    Rust Changes: If you modify expo-search-core/src/lib.rs, ensure you run the appropriate build script to update the native binaries.

    Swift/Kotlin Changes: Update the module wrappers in ios/ or android/.

    TypeScript: Update the definitions in src/ExpoRustySearch.ts and ensure they match the native method signatures.

    Test: Use the example/ folder to test your changes. Run npx expo run:ios or npx expo run:android from the example directory.

Code Style

    Rust: Run cargo fmt before committing.

    TypeScript: Use Prettier.

📄 License

MIT

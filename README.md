# 🚀 Easer (Gitee Release CLI) User Documentation

[![English](https://img.shields.io/badge/lang-English-blue.svg)](README.md)
[![中文](https://img.shields.io/badge/lang-中文-red.svg)](README.cn.md)

This is a command-line tool designed to create Releases in a specified Gitee repository using the Gitee API v5.

## ✨ Features

*   ✅ Create new Releases in a Gitee repository.
*   ✅ Support setting the tag, target commit/branch, name, and description for a Release.
*   ✅ Support marking a Release as a draft or pre-release.
*   ✅ Support authentication via Gitee Personal Access Tokens.
*   ✅ Support uploading Release artifacts.
*   ✅ Support multi-language message output (currently supports Chinese and English).
*   ✅ Validate tag names against semantic versioning specifications.

## 🛠️ Installation

### 📦 Using Cargo (Recommended)

If you have Rust and Cargo installed on your system, you can install it using the following command:

```bash
cargo install --git https://github.com/Emin017/easer.rs
```

Alternatively, clone this repository and run the following command in the project root directory:

```bash
cargo install --path .
```

### 💾 Download Pre-compiled Binaries

Visit the Releases page of this project (if available), download the pre-compiled binary suitable for your operating system, and place it in a directory included in your system's `PATH` environment variable.

## ▶️ Usage

```bash
easer \
  --owner <OWNER> \
  --repo <REPO> \
  --token <TOKEN> \
  [--repo-path <REPO_PATH>] \
  [--previous-tag <PREV_TAG>] \
  [--tag-name <TAG>] \
  [--name <NAME>] \
  [--body <BODY>] \
  --target-commitish <COMMITISH> \
  [--artifacts <PATH1>,<PATH2>,...] \
-  [--draft] [--prerelease] [--lang <LANG>]
+  [--draft <true|false>] [--prerelease <true|false>] [--auto-gen-notes <true|false>] [--lang <LANG>]
```

## ⚙️ Parameters

```markdown
*   `--owner <OWNER>`: **[Required]** Repo owner (user or org).
*   `--repo <REPO>`: **[Required]** Repository name.
*   `--token <TOKEN>`: **[Required]** Gitee personal access token.
*   `--repo-path <REPO_PATH>`: **[Optional]** Local path to Git repo, defaults to `.`.
*   `--previous-tag <PREV_TAG>`: **[Optional]** Last released tag for changelog.
*   `--tag-name <TAG>`: **[Optional]** Tag name for the new release.
*   `--name <NAME>`: **[Optional]** Release title.
*   `--body <BODY>`: **[Optional]** Release description.
*   `--target-commitish <COMMITISH>`: **[Required]** Branch or commit for the release.
*   `--artifacts <PATH1>,<PATH2>,...`: **[Optional]** Comma‑separated list of asset file paths.
-*   `--draft`: **[Optional]** Mark as draft (default `false`).
-*   `--prerelease`: **[Optional]** Mark as pre‑release (default `false`).
+*   `--draft <true|false>`: **[Optional]** Mark as draft (default `false`).
+*   `--prerelease <true|false>`: **[Optional]** Mark as pre‑release (default `false`).
+*   `--auto-gen-notes <true|false>`: **[Optional]** Automatic generation of release notes (default `false`).
*   `--lang <LANG>`: **[Optional]** Output language: `zh-cn` (default) or `en-us`.
```

## 📝 Examples

# 1. Auto‑generate release info and upload one artifact
```bash
easer \
  --owner "my-username" \
  --repo "my-project" \
  --token "TOKEN" \
  --repo-path "./" \
  --previous-tag "v0.1.0" \
  --target-commitish "main" \
  --artifacts "./dist/app.tar.gz"
```

# 2. Provide full metadata and upload multiple artifacts
```bash
easer \
  --owner "my-org" \
  --repo "beta-test" \
  --token "TOKEN" \
  --tag-name "v1.2.0" \
  --name "Release v1.2.0" \
  --body "This update includes x, y, z." \
  --target-commitish "develop" \
  --artifacts "./build.zip,./checksums.txt" \
  --lang "en-us"
```

## ⚠️ Important Notes

*   **Token Permissions**: Ensure the provided Gitee Personal Access Token has sufficient permissions (usually requires the `projects` scope) to create Releases and upload artifacts.
*   **Tag Format**: The tool checks if `--tag-name` conforms to semantic versioning (optional `v` prefix allowed). Invalid tag names will cause an error.
*   **File Paths**: The file paths specified in the `--artifacts` parameter must exist and be files. If a path does not exist or is not a file, it will be skipped.
*   **Network**: The tool needs access to the Gitee API (`https://gitee.com`). Ensure your network connection is stable.

## 🐛 Error Handling

If creating the Release or uploading artifacts fails, the tool will output an error message, typically including the status code and details returned by the Gitee API, which helps in troubleshooting. Common reasons for errors include:
*   ❌ Invalid token or insufficient permissions (401 Unauthorized)
*   ❌ Incorrect repository or owner name (404 Not Found)
*   ❌ Tag already exists (422 Unprocessable Entity)
*   ❌ Invalid tag format
*   ❌ Failed to read local artifact file
*   ❌ Gitee API error during artifact upload
*   ❌ Network issues

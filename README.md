# 🚀 Gitee Release CLI User Documentation

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
cargo install --git https://github.com/Emin017/easer.rs # Please replace with the actual repository URL
```

Alternatively, clone this repository and run the following command in the project root directory:

```bash
cargo install --path .
```

### 💾 Download Pre-compiled Binaries

Visit the Releases page of this project (if available), download the pre-compiled binary suitable for your operating system, and place it in a directory included in your system's `PATH` environment variable.

## ▶️ Usage

```bash
easer --owner <OWNER> --repo <REPO> --token <TOKEN> --tag-name <TAG> --target-commitish <COMMITISH> --name <NAME> --body <BODY> [--artifacts <PATH1>,<PATH2>,...] [OPTIONS]
```

## ⚙️ Parameter Details

*   `--owner <OWNER>`: **[Required]** The username or organization name that owns the repository.
*   `--repo <REPO>`: **[Required]** The name of the repository.
*   `--token <TOKEN>`: **[Required]** Your Gitee Personal Access Token. Requires permission to create Releases. Visit [Gitee Settings](https://gitee.com/profile/personal_access_tokens) to generate a token.
*   `--tag-name <TAG>`: **[Required]** The tag name for the Release to be created. It is recommended to follow semantic versioning (e.g., `v1.0.0`, `1.0.0`). The tool will attempt to validate its format.
*   `--target-commitish <COMMITISH>`: **[Required]** The Git commit SHA, branch name, or tag name the Release is based on (e.g., `main`, `master`, `develop`, `commit-sha`).
*   `--name <NAME>`: **[Required]** The title or name of the Release.
*   `--body <BODY>`: **[Required]** The detailed description of the Release. Supports Markdown format.
*   `--artifacts <PATH1>,<PATH2>,...`: **[Optional]** A comma-separated list of paths to the asset files to upload. Example: `--artifacts build.zip,checksums.txt`.
*   `--draft`: **[Optional]** Mark this Release as a draft. Draft Releases are not public and are only visible to repository members. Defaults to `false`.
*   `--prerelease`: **[Optional]** Mark this Release as a pre-release. Typically used for beta or release candidate versions. Defaults to `false`.
*   `--lang <LANG>`: **[Optional]** Set the language for the tool's output messages. Supports `zh-cn` (Simplified Chinese, default) and `en-us` (US English).

## 📝 Examples

Create a formal Release named "v1.0.0 Release" with the tag `v1.0.0`, based on the `main` branch, include a description, and upload two artifacts:

```bash
easer \
    --owner "my-username" \
    --repo "my-awesome-project" \
    --token "YOUR_GITEE_TOKEN" \
    --tag-name "v1.0.0" \
    --target-commitish "main" \
    --name "v1.0.0 Release" \
    --body "This is the first stable release of our project!\n\nIncludes the following updates:\n- Feature A\n- Fix B" \
    --artifacts "./dist/my-app-linux.tar.gz,./dist/my-app-windows.zip"
```

Create a pre-release draft Release and use English messages:

```bash
easer \
    --owner "my-org" \
    --repo "beta-test" \
    --token "YOUR_GITEE_TOKEN" \
    --tag-name "v0.1.0-beta.1" \
    --target-commitish "develop" \
    --name "Beta Release 1" \
    --body "This is a beta release for testing purposes." \
    --draft \
    --prerelease \
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

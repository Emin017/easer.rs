# 🚀 easer (Gitee Release CLI) 用户文档

[![English](https://img.shields.io/badge/lang-English-blue.svg)](README.en.md)
[![中文](https://img.shields.io/badge/lang-中文-red.svg)](README.md)

这是一个命令行工具，用于通过 Gitee API v5 在指定的 Gitee 仓库中创建 Release。

## ✨ 功能

*   ✅ 在 Gitee 仓库创建新的 Release。
*   ✅ 支持设置 Release 的标签、目标 Commit/分支、名称和描述。
*   ✅ 支持将 Release 标记为草稿或预发布版本。
*   ✅ 支持通过 Gitee 个人访问令牌进行认证。
*   ✅ 支持上传 Release 附件 (artifacts)。
*   ✅ 支持多语言消息输出（当前支持中文和英文）。
*   ✅ 验证标签名称是否符合语义化版本规范。

## 🛠️ 安装

### 📦 使用 Cargo （推荐）

如果你的系统安装了 Rust 和 Cargo，可以通过以下命令安装：

```bash
cargo install --git https://github.com/Emin017/easer.rs
```

或者，克隆本仓库后在项目根目录运行：

```bash
cargo install --path .
```

### 💾 下载预编译二进制文件

访问本项目的 Releases 页面（如果提供），下载适合你操作系统的预编译二进制文件，并将其放置在你的 `PATH` 环境变量所包含的目录中。

## ▶️ 用法

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
  [--draft] [--prerelease] [--lang <LANG>]
```

## ⚙️ 参数详解

*   `--owner <OWNER>`: **[必需]** 仓库所属的用户或组织名称
*   `--repo <REPO>`: **[必需]** 仓库名称
*   `--token <TOKEN>`: **[必需]** Gitee 个人访问令牌
*   `--repo-path <REPO_PATH>`: **[可选]** 本地 Git 仓库路径，默认为当前目录（`.`）。用于读取提交和生成 CHANGELOG
*   `--previous-tag <PREV_TAG>`: **[可选]** 上一个已发布的 tag，用于生成变更日志。如果不传，会自动查找最近的 tag
*   `--tag-name <TAG>`: **[可选]** 要创建的 Release 的标签名称（如 `v1.0.0`）
*   `--name <NAME>`: **[可选]** Release 的标题
*   `--body <BODY>`: **[可选]** Release 的描述，支持 Markdown
*   —— 当 `--tag-name`/`--name`/`--body` 任意一项不传时，工具会根据 Conventional Commits 规范自动生成对应信息
*   `--target-commitish <COMMITISH>`: **[必需]** Release 基于的分支或提交（如 `main`）
*   `--artifacts <PATH1>,<PATH2>,...`: **[可选]** 要上传的附件路径列表，逗号分隔
*   `--draft`: **[可选]** 将 Release 标记为草稿，默认为 `false`
*   `--prerelease`: **[可选]** 将 Release 标记为预发布，默认为 `false`
*   `--lang <LANG>`: **[可选]** 输出语言，支持 `zh-cn`（默认）和 `en-us`

## 📝 示例

# 1. 指定本地仓库、自动生成发布信息并上传 artifact
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

# 2. 指定所有信息并上传多个 artifact
```bash
easer \
  --owner "my-org" \
  --repo "beta-test" \
  --token "TOKEN" \
  --tag-name "v1.2.0" \
  --name "Release v1.2.0" \
  --body "本次更新包含 x、y、z。" \
  --target-commitish "develop" \
  --artifacts "./build.zip,./checksums.txt" \
  --lang "en-us"
```

## ⚠️ 注意事项

*   **令牌权限**: 确保提供的 Gitee 个人访问令牌具有足够的权限（通常需要 `projects` 范围）来创建 Release 和上传附件。
*   **标签格式**: 工具会检查 `--tag-name` 是否符合语义化版本规范（允许可选的 `v` 前缀）。无效的标签名会导致错误。
*   **文件路径**: `--artifacts` 参数指定的文件路径必须存在且为文件。如果文件不存在或不是文件，将被跳过。
*   **网络**: 工具需要访问 Gitee API (`https://gitee.com`)。请确保网络连接正常。

## 🐛 错误处理

如果创建 Release 或上传附件失败，工具会输出错误信息，通常包含 Gitee API 返回的状态码和详细信息，有助于排查问题。常见的错误原因包括：
*   ❌ 无效的令牌或权限不足 (401 Unauthorized)
*   ❌ 仓库或所有者名称错误 (404 Not Found)
*   ❌ 标签已存在 (422 Unprocessable Entity)
*   ❌ 无效的标签格式
*   ❌ 读取本地附件文件失败
*   ❌ 上传附件时 Gitee API 返回错误
*   ❌ 网络问题

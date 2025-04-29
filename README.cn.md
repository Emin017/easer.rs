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

### 📦 使用 Cargo (推荐)

如果你的系统安装了 Rust 和 Cargo，可以通过以下命令安装：

```bash
cargo install --git https://github.com/Emin017/easer.rs # 请替换为实际的仓库地址
```

或者，克隆本仓库后在项目根目录运行：

```bash
cargo install --path .
```

### 💾 下载预编译二进制文件

访问本项目的 Releases 页面（如果提供），下载适合你操作系统的预编译二进制文件，并将其放置在你的 `PATH` 环境变量所包含的目录中。

## ▶️ 用法

```bash
easer --owner <OWNER> --repo <REPO> --token <TOKEN> --tag-name <TAG> --target-commitish <COMMITISH> --name <NAME> --body <BODY> [--artifacts <PATH1>,<PATH2>,...] [OPTIONS]
```

## ⚙️ 参数详解

*   `--owner <OWNER>`: **[必需]** 仓库所属的用户或组织名称。
*   `--repo <REPO>`: **[必需]** 仓库名称。
*   `--token <TOKEN>`: **[必需]** 你的 Gitee 个人访问令牌。需要有创建 Release 的权限。请访问 [Gitee 设置](https://gitee.com/profile/personal_access_tokens) 生成令牌。
*   `--tag-name <TAG>`: **[必需]** 要创建的 Release 的标签名称。建议遵循语义化版本规范（例如 `v1.0.0`, `1.0.0`）。工具会尝试验证其格式。
*   `--target-commitish <COMMITISH>`: **[必需]** Release 基于的 Git Commit SHA、分支名或标签名（例如 `main`, `master`, `develop`, `commit-sha`）。
*   `--name <NAME>`: **[必需]** Release 的标题或名称。
*   `--body <BODY>`: **[必需]** Release 的详细描述。支持 Markdown 格式。
*   `--artifacts <PATH1>,<PATH2>,...`: **[可选]** 要上传的附件文件的路径列表，以逗号分隔。例如 `--artifacts build.zip,checksums.txt`。
*   `--draft`: **[可选]** 将此 Release 标记为草稿。草稿 Release 不会公开，只有仓库成员可见。默认为 `false`。
*   `--prerelease`: **[可选]** 将此 Release 标记为预发布版本。通常用于测试版或候选版本。默认为 `false`。
*   `--lang <LANG>`: **[可选]** 设置工具输出消息的语言。支持 `zh-cn` (简体中文，默认) 和 `en-us` (美国英语)。

## 📝 示例

创建一个名为 "v1.0.0 Release" 的正式 Release，标签为 `v1.0.0`，基于 `main` 分支，附带描述，并上传两个附件：

```bash
easer \
    --owner "my-username" \
    --repo "my-awesome-project" \
    --token "YOUR_GITEE_TOKEN" \
    --tag-name "v1.0.0" \
    --target-commitish "main" \
    --name "v1.0.0 Release" \
    --body "这是我们项目的第一个稳定版本！\n\n包含以下更新：\n- 功能 A\n- 修复 B" \
    --artifacts "./dist/my-app-linux.tar.gz,./dist/my-app-windows.zip"
```

创建一个预发布的草稿 Release，并使用英文消息：

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

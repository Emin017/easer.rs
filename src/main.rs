use clap::Parser;
use reqwest::header;
use semver::Version;
use serde::Serialize;
use std::error::Error;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[clap(about = "Gitee release CLI tool for creating releases")]
struct Args {
    #[clap(long, help = "Repository owner")]
    owner: String,
    #[clap(long, help = "Repository name")]
    repo: String,
    #[clap(long, help = "Gitee personal access token")]
    token: String,
    #[clap(long, help = "Tag name (e.g., v1.0.0)")]
    tag_name: String,
    #[clap(long, help = "Target commit or branch (e.g., main)")]
    target_commitish: String,
    #[clap(long, help = "Release name")]
    name: String,
    #[clap(long, help = "Release description")]
    body: String,
    #[clap(long, default_value = "false", help = "Is draft release")]
    draft: bool,
    #[clap(long, default_value = "false", help = "Is prerelease")]
    prerelease: bool,
    #[clap(long, default_value = "zh-cn", value_parser = clap::value_parser!(String), help = "Language for messages (e.g., en-us, zh-cn)")]
    lang: String,
}

#[derive(Serialize)]
struct Release {
    tag_name: String,
    target_commitish: String,
    name: String,
    body: String,
    draft: bool,
    prerelease: bool,
}

async fn create_release(args: Args, api_base_url: Option<&str>) -> Result<(), Box<dyn Error>> {
    // 添加 api_base_url 参数
    // --- i18n 消息定义 ---
    let msg_invalid_tag = match args.lang.as_str() {
        "en-us" => "Invalid semantic version tag name",
        _ => "无效的语义化版本标签名称",
    };
    let msg_success = match args.lang.as_str() {
        "en-us" => "Release created successfully",
        _ => "版本发布成功创建",
    };
    let msg_failure = match args.lang.as_str() {
        "en-us" => "Failed to create release",
        _ => "创建版本发布失败",
    };
    let msg_api_error = match args.lang.as_str() {
        "en-us" => "API request failed with status",
        _ => "API 请求失败，状态码",
    };
    // --- End i18n ---

    let tag_name_to_parse = args.tag_name.strip_prefix('v').unwrap_or(&args.tag_name);
    if Version::parse(tag_name_to_parse).is_err() {
        let err_msg = format!("{}: {}", msg_invalid_tag, args.tag_name);
        error!("{}", err_msg);
        return Err(err_msg.into());
    }
    let release = Release {
        tag_name: args.tag_name.clone(),
        target_commitish: args.target_commitish.clone(),
        name: args.name.clone(),
        body: args.body.clone(),
        draft: args.draft,
        prerelease: args.prerelease,
    };

    let client = reqwest::Client::new();
    let base_url = api_base_url.unwrap_or("https://gitee.com");
    let url = format!(
        "{}/api/v5/repos/{}/{}/releases",
        base_url, args.owner, args.repo
    );

    info!("Sending request to Gitee API: {}", url);

    let res = client
        .post(&url)
        .header(header::AUTHORIZATION, format!("token {}", args.token))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT, "application/json")
        .json(&release)
        .send()
        .await?;

    if res.status().is_success() {
        let html_url = format!(
            "https://gitee.com/{}/{}/releases/{}",
            args.owner, args.repo, args.tag_name
        );
        info!("{}: {}", msg_success, html_url);
    } else {
        let status = res.status();
        let error_text = match res.text().await {
            Ok(text) => text,
            Err(e) => {
                error!("Failed to read error response body: {}", e);
                String::from("Could not read error body")
            }
        };
        error!(
            status = status.as_str(),
            details = error_text.as_str(),
            "{}: {}",
            msg_failure,
            status
        );
        return Err(format!("{}: {} - {}", msg_api_error, status, error_text).into());
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Init tracing subscriber
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO) // Set the log level to INFO
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    let args = Args::parse();
    create_release(args, None).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    fn default_args() -> Args {
        Args {
            owner: "test_owner".to_string(),
            repo: "test_repo".to_string(),
            token: "test_token".to_string(),
            tag_name: "v1.0.0".to_string(),
            target_commitish: "main".to_string(),
            name: "Test Release".to_string(),
            body: "This is a test release.".to_string(),
            draft: false,
            prerelease: false,
            lang: "zh-cn".to_string(),
        }
    }

    #[tokio::test]
    async fn test_create_release_success() {
        let mut server = Server::new_async().await;
        let args = default_args();
        let api_path = format!("/api/v5/repos/{}/{}/releases", args.owner, args.repo);

        let mock = server
            .mock("POST", api_path.as_str())
            .with_status(201) // Created
            .with_header("content-type", "application/json")
            .with_body(r#"{"id": 1, "tag_name": "v1.0.0"}"#)
            .create_async()
            .await;

        let result = create_release(args, Some(&server.url())).await;

        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_create_release_api_error() {
        let mut server = Server::new_async().await;
        let args = default_args();
        let api_path = format!("/api/v5/repos/{}/{}/releases", args.owner, args.repo);

        let mock = server
            .mock("POST", api_path.as_str())
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(r#"{"message": "Unauthorized"}"#)
            .create_async()
            .await;

        let result = create_release(args, Some(&server.url())).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("API 请求失败，状态码: 401 Unauthorized - {\"message\": \"Unauthorized\"}"));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_create_release_invalid_tag() {
        let args = Args {
            tag_name: "invalid-tag".to_string(),
            ..default_args()
        };

        let result = create_release(args, None).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("无效的语义化版本标签名称: invalid-tag"));
    }

    #[tokio::test]
    async fn test_create_release_success_en_us() {
        let mut server = Server::new_async().await;
        let args = Args {
            lang: "en-us".to_string(),
            ..default_args()
        };
        let api_path = format!("/api/v5/repos/{}/{}/releases", args.owner, args.repo);

        let mock = server
            .mock("POST", api_path.as_str())
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id": 1, "tag_name": "v1.0.0"}"#)
            .create_async()
            .await;

        let result = create_release(args, Some(&server.url())).await;

        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_create_release_api_error_en_us() {
        let mut server = Server::new_async().await;
        let args = Args {
            lang: "en-us".to_string(),
            ..default_args()
        };
        let api_path = format!("/api/v5/repos/{}/{}/releases", args.owner, args.repo);

        let mock = server
            .mock("POST", api_path.as_str())
            .with_status(404) // Not Found
            .with_header("content-type", "application/json")
            .with_body(r#"{"message": "Not Found"}"#)
            .create_async()
            .await;

        let result = create_release(args, Some(&server.url())).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(
            "API request failed with status: 404 Not Found - {\"message\": \"Not Found\"}"
        ));
        mock.assert_async().await;
    }
}

use crate::args::Args;
use crate::conventional::generate_release_info;
use reqwest::{header, multipart};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;
use tokio::fs;
use tracing::{error, info, warn};

#[derive(Serialize)]
struct Release {
    tag_name: String,
    target_commitish: String,
    name: String,
    body: String,
    draft: bool,
    prerelease: bool,
}

#[derive(Deserialize)]
struct GiteeReleaseResponse {
    id: i64,
    html_url: Option<String>,
}

struct Messages<'a> {
    invalid_tag: &'a str,
    success: &'a str,
    failure: &'a str,
    api_error: &'a str,
    upload_start: &'a str,
    upload_success: &'a str,
    upload_failure: &'a str,
    file_read_error: &'a str,
}

impl<'a> Messages<'a> {
    fn new(lang: &str) -> Self {
        match lang {
            "en-us" => Messages {
                invalid_tag: "Invalid semantic version tag name",
                success: "Release created successfully",
                failure: "Failed to create release",
                api_error: "API request failed with status",
                upload_start: "Uploading artifact",
                upload_success: "Successfully uploaded artifact",
                upload_failure: "Failed to upload artifact",
                file_read_error: "Failed to read artifact file",
            },
            _ => Messages {
                // Default to zh-cn
                invalid_tag: "无效的语义化版本标签名称",
                success: "版本发布成功创建",
                failure: "创建版本发布失败",
                api_error: "API 请求失败，状态码",
                upload_start: "开始上传 artifact",
                upload_success: "成功上传 artifact",
                upload_failure: "上传 artifact 失败",
                file_read_error: "读取 artifact 文件失败",
            },
        }
    }
}

pub async fn create_release(args: Args, api_base_url: Option<&str>) -> Result<(), Box<dyn Error>> {
    let messages = Messages::new(&args.lang);

    let tag_name: String;
    let release_name: String;
    let release_body: String;

    if args.auto_gen_notes {
        info!("Auto-generating release notes...");
        let info = generate_release_info(
            &args.repo_path,
            args.previous_tag.as_ref(),
            &args.target_commitish,
            args.tag_name.as_deref(),
        )?;
        tag_name = info.tag_name;
        release_name = info.name;
        release_body = info.body;
    } else {
        tag_name = args.tag_name.clone().unwrap_or_default();
        release_name = args.name.clone().unwrap_or_default();
        release_body = args.body.clone().unwrap_or_default();
        [&tag_name, &release_name, &release_body]
            .iter()
            .filter(|s| s.is_empty())
            .for_each(|_| {
                error!("Tag name, release name, and body cannot be empty");
            });
    }

    let tag_name_to_parse = tag_name.strip_prefix('v').unwrap_or(&tag_name);
    if Version::parse(tag_name_to_parse).is_err() {
        let err_msg = format!("{}: {}", messages.invalid_tag, tag_name);
        error!("{}", err_msg);
        return Err(err_msg.into());
    }

    let release = Release {
        tag_name: tag_name.clone(),
        target_commitish: args.target_commitish.clone(),
        name: release_name.clone(),
        body: release_body.clone(),
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

    let release_id: i64;

    if res.status().is_success() {
        let response_body = res.text().await?;
        let release_response: GiteeReleaseResponse = match serde_json::from_str(&response_body) {
            Ok(resp) => resp,
            Err(e) => {
                error!("Failed to parse release creation response: {}", e);
                return Err(format!("Failed to parse release creation response: {}", e).into());
            }
        };
        release_id = release_response.id;
        if let Some(url) = release_response.html_url {
            info!("{}: {}", messages.success, url);
        } else {
            info!("{}", messages.success);
        }

        if let Some(artifact_paths) = args.artifacts {
            for artifact_path_str in artifact_paths {
                let artifact_path = Path::new(&artifact_path_str);
                if !artifact_path.is_file() {
                    warn!(
                        "Artifact path is not a file or does not exist, skipping: {}",
                        artifact_path_str
                    );
                    continue;
                }

                let filename = match artifact_path.file_name() {
                    Some(name) => name.to_string_lossy().to_string(),
                    None => {
                        warn!(
                            "Could not get filename for artifact, skipping: {}",
                            artifact_path_str
                        );
                        continue;
                    }
                };

                info!("{}: {}", messages.upload_start, filename);

                let file_content = match fs::read(&artifact_path).await {
                    Ok(content) => content,
                    Err(e) => {
                        error!(
                            "{}: {} - {}",
                            messages.file_read_error, artifact_path_str, e
                        );
                        continue;
                    }
                };

                let upload_url = format!(
                    "{}/api/v5/repos/{}/{}/releases/{}/attach_files",
                    base_url, args.owner, args.repo, release_id
                );

                let file_part = multipart::Part::bytes(file_content)
                    .file_name(filename.clone())
                    .mime_str("application/octet-stream")?;

                let form = multipart::Form::new()
                    .text("access_token", args.token.clone())
                    .part("file", file_part);

                info!("Uploading to: {}", upload_url);

                let upload_res = client
                    .post(&upload_url)
                    .header(header::AUTHORIZATION, format!("token {}", args.token))
                    .header(header::ACCEPT, "application/json")
                    .multipart(form)
                    .send()
                    .await;

                match upload_res {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            info!("{}: {}", messages.upload_success, filename);
                        } else {
                            let status = resp.status();
                            let error_text = match resp.text().await {
                                Ok(text) => text,
                                Err(e) => {
                                    error!("Failed to read upload error response body: {}", e);
                                    String::from("Could not read error body")
                                }
                            };
                            error!(
                                status = status.as_str(),
                                details = error_text.as_str(),
                                filename = filename.as_str(),
                                "{}: {} - Status: {}, Details: {}",
                                messages.upload_failure,
                                filename,
                                status,
                                error_text
                            );
                        }
                    }
                    Err(e) => {
                        error!(
                            filename = filename.as_str(),
                            error = e.to_string().as_str(),
                            "{}: {}",
                            messages.upload_failure,
                            filename
                        );
                    }
                }
            }
        }
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
            messages.failure,
            status
        );
        return Err(format!("{}: {} - {}", messages.api_error, status, error_text).into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::Args;
    use mockito::Server;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn default_args() -> Args {
        Args {
            owner: "test_owner".to_string(),
            repo: "test_repo".to_string(),
            token: "test_token".to_string(),
            repo_path: ".".to_string(),
            previous_tag: None,
            tag_name: Some("v1.0.0".to_string()),
            name: Some("Test Release".to_string()),
            body: Some("This is a test release.".to_string()),
            target_commitish: "main".to_string(),
            draft: false,
            prerelease: false,
            lang: "zh-cn".to_string(),
            artifacts: None,
            auto_gen_notes: false,
        }
    }

    #[tokio::test]
    async fn test_create_release_success() {
        let mut server = Server::new_async().await;
        let args = default_args();
        let api_path = format!("/api/v5/repos/{}/{}/releases", args.owner, args.repo);

        let mock = server
            .mock("POST", api_path.as_str())
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id": 1, "tag_name": "v1.0.0", "html_url": "http://example.com/release/v1.0.0"}"#)
            .create_async()
            .await;

        let result = create_release(args, Some(&server.url())).await;

        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_create_release_with_artifacts_success() {
        let mut server = Server::new_async().await;
        let dir = tempdir().unwrap();
        let file_path1 = dir.path().join("artifact1.zip");
        let file_path2 = dir.path().join("artifact2.txt");
        File::create(&file_path1)
            .unwrap()
            .write_all(b"zip content")
            .unwrap();
        File::create(&file_path2)
            .unwrap()
            .write_all(b"text content")
            .unwrap();

        let args = Args {
            artifacts: Some(vec![
                file_path1.to_str().unwrap().to_string(),
                file_path2.to_str().unwrap().to_string(),
            ]),
            ..default_args()
        };
        let release_api_path = format!("/api/v5/repos/{}/{}/releases", args.owner, args.repo);
        let release_id = 123;

        let tag = args.tag_name.clone().unwrap();
        let release_mock = server
            .mock("POST", release_api_path.as_str())
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(format!(
                r#"{{"id": {}, "tag_name": "{}", "html_url": "http://example.com/release/{}"}}"#,
                release_id, tag, tag
            ))
            .create_async()
            .await;

        let upload_path = format!(
            "/api/v5/repos/{}/{}/releases/{}/attach_files",
            args.owner, args.repo, release_id
        );

        let upload_mock1 = server
            .mock("POST", upload_path.as_str())
            .match_header("Authorization", format!("token {}", args.token).as_str())
            .match_header("Accept", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{\"name\": \"artifact1.zip\", \"url\": \"...\"}"#)
            .create_async()
            .await;

        let upload_mock2 = server
            .mock("POST", upload_path.as_str())
            .match_header("Authorization", format!("token {}", args.token).as_str())
            .match_header("Accept", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{\"name\": \"artifact2.txt\", \"url\": \"...\"}"#)
            .create_async()
            .await;

        let result = create_release(args, Some(&server.url())).await;

        assert!(result.is_ok(), "create_release failed: {:?}", result.err());
        release_mock.assert_async().await;
        upload_mock1.assert_async().await;
        upload_mock2.assert_async().await;
    }

    #[tokio::test]
    async fn test_create_release_success_no_html_url() {
        let mut server = Server::new_async().await;
        let args = default_args();
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
            tag_name: Some("invalid-tag".to_string()),
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
            .with_body(r#"{"id": 1, "tag_name": "v1.0.0", "html_url": "http://example.com/release/v1.0.0"}"#)
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
            .with_status(404)
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

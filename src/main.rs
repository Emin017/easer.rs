use clap::Parser;
use reqwest::{header, multipart}; // Import multipart
use semver::Version;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;
use tokio::fs;
use tracing::{error, info, warn, Level};
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
    #[clap(long, help = "Paths to asset files to upload", value_delimiter = ',')]
    artifacts: Option<Vec<String>>,
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

// Add struct to deserialize Gitee API response for release creation
#[derive(Deserialize)]
struct GiteeReleaseResponse {
    id: i64,
    html_url: Option<String>, // Make html_url optional
}

async fn create_release(args: Args, api_base_url: Option<&str>) -> Result<(), Box<dyn Error>> {
    // 添加 api_base_url 参数
    // --- i18n 消 Messages ---
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
    let msg_upload_start = match args.lang.as_str() {
        "en-us" => "Uploading artifact",
        _ => "开始上传 artifact",
    };
    let msg_upload_success = match args.lang.as_str() {
        "en-us" => "Successfully uploaded artifact",
        _ => "成功上传 artifact",
    };
    let msg_upload_failure = match args.lang.as_str() {
        "en-us" => "Failed to upload artifact",
        _ => "上传 artifact 失败",
    };
    let msg_file_read_error = match args.lang.as_str() {
        "en-us" => "Failed to read artifact file",
        _ => "读取 artifact 文件失败",
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

    let release_id: i64; // Variable to store the release ID

    if res.status().is_success() {
        // Parse the response to get the release ID and html_url
        let response_body = res.text().await?;
        let release_response: GiteeReleaseResponse = match serde_json::from_str(&response_body) {
            Ok(resp) => resp,
            Err(e) => {
                error!("Failed to parse release creation response: {}", e);
                return Err(format!("Failed to parse release creation response: {}", e).into());
            }
        };
        release_id = release_response.id;
        // Log success message, handling optional html_url
        if let Some(url) = release_response.html_url {
            info!("{}: {}", msg_success, url);
        } else {
            info!("{}", msg_success);
        }

        // --- Artifact Upload Logic ---
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

                info!("{}: {}", msg_upload_start, filename);

                let file_content = match fs::read(&artifact_path).await {
                    Ok(content) => content,
                    Err(e) => {
                        error!("{}: {} - {}", msg_file_read_error, artifact_path_str, e);
                        continue;
                    }
                };

                // Use the /attach_files endpoint
                let upload_url = format!(
                    "{}/api/v5/repos/{}/{}/releases/{}/attach_files",
                    base_url, args.owner, args.repo, release_id
                );

                // Create multipart form
                let file_part = multipart::Part::bytes(file_content)
                    .file_name(filename.clone())
                    .mime_str("application/octet-stream")?; // Set MIME type for the file part

                let form = multipart::Form::new()
                    .text("access_token", args.token.clone()) // Add access_token as text part
                    .part("file", file_part); // Add file part named "file"

                info!("Uploading to: {}", upload_url);

                let upload_res = client
                    .post(&upload_url)
                    // Authorization header might still be needed depending on Gitee's specific implementation for this endpoint,
                    // but the token is primarily sent in the form data now. Let's keep it for safety.
                    .header(header::AUTHORIZATION, format!("token {}", args.token))
                    .header(header::ACCEPT, "application/json") // Expect JSON response
                    .multipart(form) // Send as multipart
                    .send()
                    .await;

                match upload_res {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            // Gitee attach_files might return 200 OK or 201 Created on success
                            info!("{}: {}", msg_upload_success, filename);
                            // Optionally log the response body if needed
                            // let response_body = resp.text().await?;
                            // info!("Upload response: {}", response_body);
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
                                msg_upload_failure,
                                filename,
                                status,
                                error_text // Include details in the main message
                            );
                            // Log error but continue uploading other artifacts
                        }
                    }
                    Err(e) => {
                        error!(
                            filename = filename.as_str(),
                            error = e.to_string().as_str(),
                            "{}: {}",
                            msg_upload_failure,
                            filename
                        );
                        // Log error but continue uploading other artifacts
                    }
                }
            }
        }
        // --- End Artifact Upload Logic ---
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
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

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
            artifacts: None, // Default to no artifacts
        }
    }

    #[tokio::test]
    async fn test_create_release_success() {
        let mut server = Server::new_async().await;
        let args = default_args();
        let api_path = format!("/api/v5/repos/{}/{}/releases", args.owner, args.repo);

        // Mock response still includes html_url, which is fine
        let mock = server
            .mock("POST", api_path.as_str())
            .with_status(201) // Created
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
        let file_content1 = b"zip content";
        let file_content2 = b"text content";
        File::create(&file_path1)
            .unwrap()
            .write_all(file_content1)
            .unwrap();
        File::create(&file_path2)
            .unwrap()
            .write_all(file_content2)
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

        let release_mock = server
            .mock("POST", release_api_path.as_str())
            .with_status(201)
            .with_header("content-type", "application/json")
            // Corrected JSON format string: removed unnecessary backslashes before quotes inside the raw string
            .with_body(format!(
                r#"{{"id": {}, "tag_name": "{}", "html_url": "http://example.com/release/{}"}}"#,
                release_id, args.tag_name, args.tag_name
            ))
            .create_async()
            .await;

        // Mock artifact uploads using the /attach_files endpoint and multipart/form-data
        let upload_path = format!(
            "/api/v5/repos/{}/{}/releases/{}/attach_files",
            args.owner, args.repo, release_id
        );

        // Mock for artifact1.zip
        let upload_mock1 = server
            .mock("POST", upload_path.as_str())
            .match_header("Authorization", format!("token {}", args.token).as_str())
            .match_header("Accept", "application/json")
            // We can't easily match the exact multipart body with basic mockito matchers.
            // We rely on the fact that reqwest builds the correct multipart request.
            .with_status(200) // attach_files often returns 200 OK
            .with_header("content-type", "application/json")
            .with_body(r#"{\"name\": \"artifact1.zip\", \"url\": \"...\"}"#) // Example success response
            .create_async()
            .await;

        // Mock for artifact2.txt
        let upload_mock2 = server
            .mock("POST", upload_path.as_str()) // Same path for the second file
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
        // Assert that both upload mocks were called exactly once
        upload_mock1.assert_async().await;
        upload_mock2.assert_async().await;
    }

    // Add a test case where html_url is missing in the response
    #[tokio::test]
    async fn test_create_release_success_no_html_url() {
        let mut server = Server::new_async().await;
        let args = default_args();
        let api_path = format!("/api/v5/repos/{}/{}/releases", args.owner, args.repo);

        // Mock response *without* html_url
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

        // Mock response still includes html_url
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

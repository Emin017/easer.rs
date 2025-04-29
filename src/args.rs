use clap::Parser;

#[derive(Parser, Debug)] // 添加 Debug trait 以便在 main 中打印
#[clap(about = "Gitee release CLI tool for creating releases")]
pub struct Args {
    #[clap(long, help = "Repository owner")]
    pub owner: String,
    #[clap(long, help = "Repository name")]
    pub repo: String,
    #[clap(long, help = "Gitee personal access token")]
    pub token: String,
    #[clap(long, help = "Tag name (e.g., v1.0.0)")]
    pub tag_name: String,
    #[clap(long, help = "Target commit or branch (e.g., main)")]
    pub target_commitish: String,
    #[clap(long, help = "Release name")]
    pub name: String,
    #[clap(long, help = "Release description")]
    pub body: String,
    #[clap(long, default_value = "false", help = "Is draft release")]
    pub draft: bool,
    #[clap(long, default_value = "false", help = "Is prerelease")]
    pub prerelease: bool,
    #[clap(long, default_value = "zh-cn", value_parser = clap::value_parser!(String), help = "Language for messages (e.g., en-us, zh-cn)")]
    pub lang: String,
    #[clap(long, help = "Paths to asset files to upload", value_delimiter = ',')]
    pub artifacts: Option<Vec<String>>,
}

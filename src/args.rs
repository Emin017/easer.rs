use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(about = "Gitee release CLI tool for creating releases")]
pub struct Args {
    #[clap(long, help = "Repository owner")]
    pub owner: String,
    #[clap(long, help = "Repository name")]
    pub repo: String,
    #[clap(long, help = "Gitee personal access token")]
    pub token: String,
    #[clap(long, default_value = ".", help = "Path to repository to analyze")]
    pub repo_path: String,
    #[clap(long, help = "Previous tag to start analysis from")]
    pub previous_tag: Option<String>,
    #[clap(long, help = "Tag name (e.g., v1.0.0), optional for auto-generation")]
    pub tag_name: Option<String>,
    #[clap(long, help = "Release name, optional for auto-generation")]
    pub name: Option<String>,
    #[clap(long, help = "Release description, optional for auto-generation")]
    pub body: Option<String>,
    #[clap(long, help = "Target commit or branch (e.g., main)")]
    pub target_commitish: String,
    #[clap(long, default_value = "false", help = "Is draft release")]
    pub draft: bool,
    #[clap(long, default_value = "false", help = "Is prerelease")]
    pub prerelease: bool,
    #[clap(long, default_value = "zh-cn", value_parser = clap::value_parser!(String), help = "Language for messages (e.g., en-us, zh-cn)")]
    pub lang: String,
    #[clap(long, help = "Paths to asset files to upload", value_delimiter = ',')]
    pub artifacts: Option<Vec<String>>,
    #[clap(
        long,
        default_value = "false",
        help = "Is automatic generation of release notes"
    )]
    pub auto_gen_notes: bool,
}

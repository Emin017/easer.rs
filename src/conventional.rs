use git2::{Cred, FetchOptions, RemoteCallbacks, Repository, Sort};
use semver::Version;
use std::error::Error;

pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: String,
    pub body: String,
}

/// Generate ReleaseInfo using Conventional Commits
pub fn generate_release_info(
    repo_path: &str,
    previous_tag: Option<&String>,
    target: &str,
    manual_version: Option<&str>,
) -> Result<ReleaseInfo, Box<dyn Error>> {
    let repo = Repository::open(repo_path)?;

    let mut callbacks = RemoteCallbacks::new();
    // Support SSH and HTTPS: use SSH agent for git@ URLs, credential helper for HTTP(S)
    let config = repo.config()?;
    callbacks.credentials(move |url, username_from_url, _| {
        if url.starts_with("http") {
            // HTTP(S) auth via git credential helper
            Cred::credential_helper(&config, url, username_from_url)
        } else {
            // SSH auth via ssh-agent
            Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        }
    });
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let mut remote = repo.find_remote("origin")?;
    remote.fetch(&["refs/tags/*:refs/tags/*"], Some(&mut fo), None)?;

    // Prepare base URL for commit links
    let origin_url = remote.url().unwrap_or("").to_string();
    let base_url = if origin_url.starts_with("git@") {
        if let Some((host, path)) = origin_url.trim_start_matches("git@").split_once(':') {
            format!("https://{}/{}", host, path.trim_end_matches(".git"))
        } else {
            origin_url.clone()
        }
    } else if origin_url.starts_with("http") {
        origin_url.trim_end_matches(".git").to_string()
    } else {
        origin_url.clone()
    };

    // Collect semver tags
    let tag_names = repo.tag_names(None)?;
    let mut versions = vec![];
    for name_opt in tag_names.iter() {
        if let Some(name) = name_opt {
            if let Some(stripped) = name.strip_prefix('v') {
                if let Ok(ver) = Version::parse(stripped) {
                    versions.push((ver, name.to_string()));
                }
            }
        }
    }
    versions.sort_by(|a, b| a.0.cmp(&b.0));
    // Determine base version
    let (base_version, base_tag) = if let Some(prev) = previous_tag {
        if let Some((ver, tag)) = versions.iter().find(|(_, t)| t == prev) {
            (ver.clone(), tag.clone())
        } else {
            return Err(format!("Previous tag {} not found", prev).into());
        }
    } else if versions.is_empty() {
        (Version::new(0, 0, 0), String::new())
    } else {
        versions.pop().unwrap()
    };

    // Walk commits since base_tag to target
    let mut revwalk = repo.revwalk()?;
    revwalk.push_range(&format!("{}..{}", base_tag, target))?;
    if base_tag.is_empty() {
        revwalk.push_ref(target)?;
    } else {
        revwalk.push_range(&format!("{}..{}", base_tag, target))?;
    }
    revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)?;

    let mut bump_major = false;
    let mut bump_minor = false;
    let mut features: Vec<(String, String)> = Vec::new();
    let mut fixes: Vec<(String, String)> = Vec::new();
    let mut breaking: Vec<(String, String)> = Vec::new();

    for oid_res in revwalk {
        let oid = oid_res?;
        let commit = repo.find_commit(oid)?;
        let msg = commit.summary().unwrap_or("").to_string();
        let sha = oid.to_string();
        // Categorize commit
        if msg.contains("BREAKING CHANGE") || msg.contains("!:") {
            breaking.push((msg.clone(), sha.clone()));
            bump_major = true;
        } else if msg.starts_with("feat") {
            features.push((msg.clone(), sha.clone()));
            if !bump_major {
                bump_minor = true;
            }
        } else if msg.starts_with("fix") {
            fixes.push((msg.clone(), sha.clone()));
        }
    }
    // Bump version or use manual version
    let next = if let Some(ver_str) = manual_version {
        // Supports "v0.2.1" or "0.2.1"
        Version::parse(ver_str.trim_start_matches('v'))?
    } else {
        let mut v = base_version.clone();
        if bump_major {
            v.major += 1;
            v.minor = 0;
            v.patch = 0;
        } else if bump_minor {
            v.minor += 1;
            v.patch = 0;
        } else {
            v.patch += 1;
        }
        v
    };

    // Construct tag and name
    let tag_name = format!("v{}", next);
    let name = format!("Release {}", next);
    // Build changelog body
    let mut body = String::new();
    if !breaking.is_empty() {
        body.push_str("## ⚠ BREAKING CHANGES\n");
        for (msg, sha) in breaking {
            let short = &sha[..7];
            body.push_str(&format!(
                "- {} ([{}]({}/commit/{}))\n",
                msg, short, base_url, sha
            ));
        }
        body.push('\n');
    }
    if !features.is_empty() {
        body.push_str("## ✨ Features\n");
        for (msg, sha) in features {
            let short = &sha[..7];
            body.push_str(&format!(
                "- {} ([{}]({}/commit/{}))\n",
                msg, short, base_url, sha
            ));
        }
        body.push('\n');
    }
    if !fixes.is_empty() {
        body.push_str("## 🐛 Bug Fixes\n");
        for (msg, sha) in fixes {
            let short = &sha[..7];
            body.push_str(&format!(
                "- {} ([{}]({}/commit/{}))\n",
                msg, short, base_url, sha
            ));
        }
        body.push('\n');
    }
    Ok(ReleaseInfo {
        tag_name,
        name,
        body,
    })
}

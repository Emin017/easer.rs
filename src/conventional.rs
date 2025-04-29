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
) -> Result<ReleaseInfo, Box<dyn Error>> {
    let repo = Repository::open(repo_path)?;

    let mut callbacks = RemoteCallbacks::new();
    // 用 SSH agent
    callbacks.credentials(|_url, username_from_url, _| {
        Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
    });
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let mut remote = repo.find_remote("origin")?;
    remote.fetch(&["refs/tags/*:refs/tags/*"], Some(&mut fo), None)?;

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
    let mut features = Vec::new();
    let mut fixes = Vec::new();
    let mut breaking = Vec::new();

    for oid_res in revwalk {
        let oid = oid_res?;
        let commit = repo.find_commit(oid)?;
        let msg = commit.summary().unwrap_or("").to_string();
        // Breaking change
        if msg.contains("BREAKING CHANGE") || msg.contains("!:") {
            breaking.push(msg.clone());
            bump_major = true;
        } else if msg.starts_with("feat") {
            features.push(msg.clone());
            if !bump_major {
                bump_minor = true;
            }
        } else if msg.starts_with("fix") {
            fixes.push(msg.clone());
        }
    }
    // Bump version
    let mut next = base_version.clone();
    if bump_major {
        next.major += 1;
        next.minor = 0;
        next.patch = 0;
    } else if bump_minor {
        next.minor += 1;
        next.patch = 0;
    } else {
        next.patch += 1;
    }
    // Construct tag and name
    let tag_name = format!("v{}", next);
    let name = format!("Release {}", next);
    // Build changelog body
    let mut body = String::new();
    if !breaking.is_empty() {
        body.push_str("## ⚠ BREAKING CHANGES\n");
        for b in breaking {
            body.push_str(&format!("- {}\n", b));
        }
        body.push('\n');
    }
    if !features.is_empty() {
        body.push_str("## ✨ Features\n");
        for f in features {
            body.push_str(&format!("- {}\n", f));
        }
        body.push('\n');
    }
    if !fixes.is_empty() {
        body.push_str("## 🐛 Bug Fixes\n");
        for f in fixes {
            body.push_str(&format!("- {}\n", f));
        }
        body.push('\n');
    }
    Ok(ReleaseInfo {
        tag_name,
        name,
        body,
    })
}

use std::fs;
use std::path::PathBuf;

pub fn skill_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    dirs.push(PathBuf::from("/etc/agntos/skills"));
    if let Ok(home) = std::env::var("HOME") {
        dirs.push(PathBuf::from(home).join(".config/agntos/skills"));
    }
    dirs
}

pub fn list_skills() -> Vec<String> {
    let mut names = Vec::new();
    for dir in skill_dirs() {
        let Ok(entries) = fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.join("SKILL.md").is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        names.push(name.to_string());
                    }
                }
            } else if path.extension().is_some_and(|e| e == "md") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    names.push(stem.to_string());
                }
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

pub fn load_skill(name: &str) -> Result<String, String> {
    for dir in skill_dirs() {
        let dir_path = dir.join(name);
        let skill_md = dir_path.join("SKILL.md");
        if skill_md.is_file() {
            return fs::read_to_string(&skill_md).map_err(|e| e.to_string());
        }
        let flat = dir.join(format!("{}.md", name));
        if flat.is_file() {
            return fs::read_to_string(&flat).map_err(|e| e.to_string());
        }
    }
    Err(format!("skill not found: {} (looked in /etc/agntos/skills, ~/.config/agntos/skills)", name))
}

pub fn skill_prompt(name: &str) -> Result<String, String> {
    let body = load_skill(name)?;
    Ok(format!(
        "Follow this skill for the user's request.\n\n--- skill:{} ---\n{}\n--- end skill ---",
        name, body.trim()
    ))
}

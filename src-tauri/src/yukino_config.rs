use std::path::PathBuf;

use crate::codex_config::{self, CodexLikeApp};

pub fn get_yukino_dir() -> PathBuf {
    codex_config::get_codex_like_config_dir(CodexLikeApp::Yukino)
}

pub fn get_yukino_auth_path() -> PathBuf {
    codex_config::get_codex_like_auth_path(CodexLikeApp::Yukino)
}

pub fn get_yukino_config_path() -> PathBuf {
    codex_config::get_codex_like_config_path(CodexLikeApp::Yukino)
}

pub fn get_yukino_memory_path() -> PathBuf {
    get_yukino_dir().join("AGENTS.md")
}

pub fn get_yukino_skills_dir() -> PathBuf {
    get_yukino_dir().join("skills")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn yukino_paths_default_to_yukino_home() {
        let home = crate::config::get_home_dir();
        assert_eq!(get_yukino_dir(), home.join(".yukino"));
        assert_eq!(
            get_yukino_auth_path().parent(),
            Some(home.join(".yukino").as_path())
        );
        assert_eq!(
            get_yukino_config_path().parent(),
            Some(home.join(".yukino").as_path())
        );
        assert_eq!(
            get_yukino_memory_path().parent(),
            Some(home.join(".yukino").as_path())
        );
        assert_eq!(
            get_yukino_skills_dir().parent(),
            Some(home.join(".yukino").as_path())
        );
    }
}

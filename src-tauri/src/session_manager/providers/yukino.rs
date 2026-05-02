use std::path::Path;

use crate::session_manager::{SessionMessage, SessionMeta};
use crate::yukino_config::get_yukino_dir;

use super::codex;

const PROVIDER_ID: &str = "yukino";
const PROVIDER_LABEL: &str = "Yukino";
const RESUME_BINARY: &str = "codex";

pub fn scan_sessions() -> Vec<SessionMeta> {
    let root = get_yukino_dir().join("sessions");
    codex::scan_sessions_from_root(&root, PROVIDER_ID, RESUME_BINARY)
}

pub fn load_messages(path: &Path) -> Result<Vec<SessionMessage>, String> {
    codex::load_messages(path)
}

pub fn delete_session(_root: &Path, path: &Path, session_id: &str) -> Result<bool, String> {
    codex::delete_session_for_provider(
        path,
        session_id,
        PROVIDER_ID,
        PROVIDER_LABEL,
        RESUME_BINARY,
    )
}

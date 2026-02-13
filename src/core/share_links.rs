use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};

#[derive(Clone, Debug)]
pub enum SharedEntry {
    File {
        name: String,
        path: PathBuf,
        file_type: String,
    },
    Text {
        name: String,
        content: String,
    },
}

#[derive(Clone, Debug)]
pub struct ShareLinkRecord {
    pub id: String,
    pub entries: Vec<SharedEntry>,
}

static SHARE_LINKS: LazyLock<RwLock<HashMap<String, ShareLinkRecord>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn create_share(entries: Vec<SharedEntry>) -> Option<String> {
    if entries.is_empty() {
        return None;
    }
    let id = uuid::Uuid::new_v4().to_string();
    let record = ShareLinkRecord {
        id: id.clone(),
        entries,
    };
    if let Ok(mut guard) = SHARE_LINKS.write() {
        guard.insert(id.clone(), record);
        prune_old_links(&mut guard);
        Some(id)
    } else {
        None
    }
}

pub fn get_share(id: &str) -> Option<ShareLinkRecord> {
    SHARE_LINKS.read().ok()?.get(id).cloned()
}

fn prune_old_links(links: &mut HashMap<String, ShareLinkRecord>) {
    if links.len() <= 32 {
        return;
    }
    let mut ids: Vec<String> = links.keys().cloned().collect();
    ids.sort_unstable();
    let target_keep = 16usize;
    for id in ids {
        if links.len() <= target_keep {
            break;
        }
        links.remove(&id);
    }
}

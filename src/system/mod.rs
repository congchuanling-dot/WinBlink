use std::collections::HashSet;
use std::env;
use std::path::Path;

use crate::common::error::WinBlinkError;
use crate::common::types::{ItemType, SearchItem};

pub fn get_installed_apps() -> Result<Vec<SearchItem>, WinBlinkError> {
    let username = env::var("USERNAME").unwrap_or_default();

    let start_menu_paths: [String; 2] = [
        r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs".to_string(),
        format!(
            r"C:\Users\{}\AppData\Roaming\Microsoft\Windows\Start Menu\Programs",
            username
        ),
    ];

    let everything = everywhere::Everything::connect().map_err(|e| {
        WinBlinkError::Everything(format!("无法连接到 Everything 服务: {e}"))
    })?;

    let path_queries: Vec<String> = start_menu_paths
        .iter()
        .map(|p| format!("\"{}\" *.lnk", p))
        .collect();
    let search_query = path_queries.join(" | ");

    everything
        .set_search(&search_query)
        .map_err(|e| WinBlinkError::Everything(format!("设置搜索条件失败: {e}")))?;

    everything
        .query(true)
        .map_err(|e| WinBlinkError::Everything(format!("执行搜索失败: {e}")))?;

    let num_results = everything.num_results();
    let mut seen: HashSet<String> = HashSet::new();
    let mut apps: Vec<SearchItem> = Vec::new();

    for i in 0..num_results {
        let lnk_path = match everything.result_full_path_and_filename(i) {
            Some(p) => p,
            None => continue,
        };

        let shortcut = match lnk::ShellLink::open(&lnk_path, 1252) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let target_path = match shortcut.link_target() {
            Some(p) => p,
            None => continue,
        };

        let target_lower = target_path.to_lowercase();
        if !target_lower.ends_with(".exe") {
            continue;
        }

        if target_lower.contains("uninstall") || target_path.contains("卸载") {
            continue;
        }

        if !seen.insert(target_path.clone()) {
            continue;
        }

        let name = shortcut
            .name_string()
            .map(|s| s.to_string())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| {
                Path::new(&target_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            });

        apps.push(SearchItem {
            id: apps.len() as u64,
            name,
            path: target_path,
            item_type: ItemType::Application,
        });
    }

    Ok(apps)
}

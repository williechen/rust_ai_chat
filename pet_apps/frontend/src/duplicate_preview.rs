use serde::Deserialize;

// 僅保留顯示所需資料，不持有 hash、警告本文或私人路徑。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroupView {
    size_bytes: u64,
    item_ids: Vec<serde::de::IgnoredAny>,
}

#[derive(Debug, Deserialize)]
pub struct DuplicatePreviewView {
    groups: Vec<DuplicateGroupView>,
    warnings: Vec<serde::de::IgnoredAny>,
}

impl DuplicatePreviewView {
    pub fn summary(&self) -> String {
        let mut text = format!(
            "SHA256 重複群組：{}；掃描警告：{}。僅供預覽，尚未移動或刪除檔案。",
            self.groups.len(),
            self.warnings.len()
        );
        for (index, group) in self.groups.iter().enumerate() {
            text.push_str(&format!(
                "\n群組 {}：每檔 {} 位元組，{} 個檔案。",
                index + 1,
                group.size_bytes,
                group.item_ids.len()
            ));
        }
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_shows_group_sizes_without_private_paths() {
        let value: DuplicatePreviewView = serde_json::from_str(
            r#"{
            "groups": [
                {"sizeBytes":4,"itemIds":["item-000000","item-000001"],
                 "digestSha256":"private-hash"},
                {"sizeBytes":0,"itemIds":["item-000002","item-000003","item-000004"]}
            ],
            "warnings":[{"relativePath":"private.txt","message":"private-error"}]
        }"#,
        )
        .unwrap();
        let text = value.summary();
        assert!(text.contains("SHA256 重複群組：2；掃描警告：1"));
        assert!(text.contains("群組 1：每檔 4 位元組，2 個檔案"));
        assert!(text.contains("群組 2：每檔 0 位元組，3 個檔案"));
        assert!(text.contains("尚未移動或刪除"));
        for private in [
            "private.txt",
            "private-error",
            "private-hash",
            "item-000000",
        ] {
            assert!(!text.contains(private));
        }
    }

    #[test]
    fn empty_preview_is_explicit_and_missing_contract_is_rejected() {
        let value: DuplicatePreviewView =
            serde_json::from_str(r#"{"groups":[],"warnings":[]}"#).unwrap();
        assert_eq!(
            value.summary(),
            "SHA256 重複群組：0；掃描警告：0。僅供預覽，尚未移動或刪除檔案。"
        );
        assert!(
            serde_json::from_str::<DuplicatePreviewView>(
                r#"{"groups":[{"itemIds":[]}],"warnings":[]}"#
            )
            .is_err()
        );
    }
}

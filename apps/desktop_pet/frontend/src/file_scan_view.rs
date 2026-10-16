use serde::Deserialize;

// 顯示層丟棄路徑、opaque ID 與警告本文。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileScanView {
    files: Vec<serde::de::IgnoredAny>,
    duplicate_size_candidates: Vec<serde::de::IgnoredAny>,
    warnings: Vec<serde::de::IgnoredAny>,
}

impl FileScanView {
    pub fn summary(&self) -> String {
        format!(
            "掃描檔案：{}；同大小候選群組：{}；警告：{}。同大小不代表內容相同；僅供預覽，尚未移動或刪除檔案。",
            self.files.len(),
            self.duplicate_size_candidates.len(),
            self.warnings.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_are_visible_but_private_values_are_discarded() {
        let value: FileScanView = serde_json::from_str(
            r#"{
            "files":[{"id":"private-id","relativePath":"private.txt","sizeBytes":3},{}],
            "duplicateSizeCandidates":[{"sizeBytes":3,"itemIds":["private-id","other-id"]}],
            "warnings":[{"relativePath":"secret.txt","message":"private-error"}]
        }"#,
        )
        .unwrap();
        let text = value.summary();
        assert!(text.contains("掃描檔案：2；同大小候選群組：1；警告：1"));
        assert!(text.contains("同大小不代表內容相同"));
        assert!(text.contains("尚未移動或刪除"));
        for private in ["private-id", "private.txt", "secret.txt", "private-error"] {
            assert!(!text.contains(private));
        }
    }

    #[test]
    fn empty_is_explicit_and_incomplete_contract_is_rejected() {
        let value: FileScanView =
            serde_json::from_str(r#"{"files":[],"duplicateSizeCandidates":[],"warnings":[]}"#)
                .unwrap();
        assert!(
            value
                .summary()
                .contains("掃描檔案：0；同大小候選群組：0；警告：0")
        );
        assert!(serde_json::from_str::<FileScanView>(r#"{"files":[],"warnings":[]}"#).is_err());
    }
}

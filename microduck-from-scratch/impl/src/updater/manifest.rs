//! release manifest：`<ver>.manifest.json`，字段 version / artifact / sha256。
//!
//! 原版 manifest 还带 minisign 签名；我们只有 sha256 完整性（D38）。
//! 解析失败 = 拒绝，不容忍缺字段——"unproven is not healthy" 的同款
//! 立场：说不清的 manifest 不是"也许能装"，是"不装"。

use serde::Deserialize;
use sha2::Digest as _;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub version: String,
    /// artifact 文件名（相对 source/ 目录，不许带路径分隔符——
    /// 否则 manifest 能让 tar 指到 source/ 外面去）。
    pub artifact: String,
    /// 64 位小写十六进制 sha256。
    pub sha256: String,
}

impl Manifest {
    /// 解析 + 形状校验。任何不符都 Err，调用方据此拒绝本次 apply。
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("read manifest {}: {e}", path.display()))?;
        let m: Manifest = serde_json::from_str(&text)
            .map_err(|e| format!("parse manifest {}: {e}", path.display()))?;
        if m.version.is_empty() {
            return Err("manifest version is empty".into());
        }
        if m.artifact.is_empty() || m.artifact.contains('/') || m.artifact.contains('\\') {
            return Err(format!("manifest artifact {:?} is not a bare file name", m.artifact));
        }
        let ok_sha = m.sha256.len() == 64
            && m.sha256.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase());
        if !ok_sha {
            return Err(format!("manifest sha256 {:?} is not 64 lowercase hex chars", m.sha256));
        }
        Ok(m)
    }

    /// 读 artifact 字节算 sha256 并与 manifest 比对。不符 → Err，
    /// 调用方保证不留任何副作用（不进 staging）。
    pub fn verify_artifact(&self, source_dir: &Path) -> Result<(), String> {
        let path = source_dir.join(&self.artifact);
        let got = sha256_file(&path)?;
        if got != self.sha256 {
            return Err(format!(
                "sha256 mismatch for {}: manifest says {}, file is {}",
                self.artifact, self.sha256, got
            ));
        }
        Ok(())
    }
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("read artifact {}: {e}", path.display()))?;
    Ok(hex_lower(&sha2::Sha256::digest(&bytes)))
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::updater::store::tests::TempDir;

    #[test]
    fn parses_well_formed_manifest() {
        let tmp = TempDir::new("manifest-ok");
        std::fs::write(tmp.path().join("a.tar"), b"hello").unwrap();
        let sha = sha256_file(&tmp.path().join("a.tar")).unwrap();
        let m = tmp.write(
            "1.0.0.manifest.json",
            &format!(r#"{{"version":"1.0.0","artifact":"a.tar","sha256":"{sha}"}}"#),
        );
        let m = Manifest::load(&m).unwrap();
        assert_eq!(m.version, "1.0.0");
        m.verify_artifact(tmp.path()).unwrap();
    }

    #[test]
    fn rejects_unparseable_and_malformed() {
        let tmp = TempDir::new("manifest-bad");
        for (name, text) in [
            ("not-json", "this is not json"),
            ("missing-sha", r#"{"version":"1.0.0","artifact":"a.tar"}"#),
            ("empty-version", r#"{"version":"","artifact":"a.tar","sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#),
            ("short-sha", r#"{"version":"1.0.0","artifact":"a.tar","sha256":"abcd"}"#),
            ("upper-sha", r#"{"version":"1.0.0","artifact":"a.tar","sha256":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"}"#),
            ("path-artifact", r#"{"version":"1.0.0","artifact":"../a.tar","sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#),
        ] {
            let p = tmp.write(name, text);
            assert!(Manifest::load(&p).is_err(), "{name} should be rejected");
        }
    }

    #[test]
    fn rejects_sha256_mismatch() {
        let tmp = TempDir::new("manifest-sha");
        std::fs::write(tmp.path().join("a.tar"), b"tampered bytes").unwrap();
        let p = tmp.write(
            "1.0.0.manifest.json",
            r#"{"version":"1.0.0","artifact":"a.tar","sha256":"0000000000000000000000000000000000000000000000000000000000000000"}"#,
        );
        let m = Manifest::load(&p).unwrap();
        let err = m.verify_artifact(tmp.path()).unwrap_err();
        assert!(err.contains("sha256 mismatch"), "{err}");
    }
}

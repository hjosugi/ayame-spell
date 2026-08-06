//! Reproducible dictionary registry resolution.

use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const LOCK_FILE: &str = "ayame-spell.lock";
const LOCK_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryLock {
    pub version: u32,
    #[serde(default, rename = "dictionary")]
    pub dictionaries: Vec<LockedDictionary>,
}

impl Default for RegistryLock {
    fn default() -> Self {
        Self {
            version: LOCK_VERSION,
            dictionaries: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedDictionary {
    pub name: String,
    pub version: String,
    pub sha256: String,
    pub file: String,
}

impl RegistryLock {
    pub fn load(root: &Path) -> anyhow::Result<Self> {
        let path = root.join(LOCK_FILE);
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(error) => return Err(error.into()),
        };
        let mut lock: Self =
            toml::from_str(&text).with_context(|| format!("cannot parse {}", path.display()))?;
        anyhow::ensure!(
            lock.version == LOCK_VERSION,
            "unsupported lockfile version {} in {}",
            lock.version,
            path.display()
        );
        lock.dictionaries
            .sort_by(|left, right| left.name.cmp(&right.name));
        Ok(lock)
    }

    pub fn save(&mut self, root: &Path) -> anyhow::Result<PathBuf> {
        self.version = LOCK_VERSION;
        self.dictionaries
            .sort_by(|left, right| left.name.cmp(&right.name));
        self.dictionaries
            .dedup_by(|left, right| left.name == right.name);
        let path = root.join(LOCK_FILE);
        let output = toml::to_string_pretty(self)?;
        std::fs::write(&path, output)
            .with_context(|| format!("cannot write {}", path.display()))?;
        Ok(path)
    }

    pub fn get(&self, name: &str) -> Option<&LockedDictionary> {
        self.dictionaries
            .iter()
            .find(|dictionary| dictionary.name == name)
    }

    pub fn upsert(&mut self, dictionary: LockedDictionary) {
        if let Some(existing) = self
            .dictionaries
            .iter_mut()
            .find(|existing| existing.name == dictionary.name)
        {
            *existing = dictionary;
        } else {
            self.dictionaries.push(dictionary);
        }
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let before = self.dictionaries.len();
        self.dictionaries
            .retain(|dictionary| dictionary.name != name);
        self.dictionaries.len() != before
    }
}

pub struct ResolvedRegistryRef {
    pub name: String,
    pub version: Option<String>,
    pub sha256: Option<String>,
    pub path: PathBuf,
}

pub fn split_reference(reference: &str) -> (&str, Option<&str>) {
    reference
        .rsplit_once('@')
        .map_or((reference, None), |(name, version)| (name, Some(version)))
}

pub fn resolve(root: &Path, reference: &str) -> anyhow::Result<ResolvedRegistryRef> {
    let (name, requested_version) = split_reference(reference);
    anyhow::ensure!(!name.is_empty(), "empty registry dictionary name");
    if let Some(version) = requested_version {
        anyhow::ensure!(!version.is_empty(), "empty registry version for `{name}`");
    }
    let lock = RegistryLock::load(root)?;
    let locked = lock
        .get(name)
        .filter(|dictionary| requested_version.is_none_or(|version| version == dictionary.version));
    let version = requested_version
        .map(str::to_string)
        .or_else(|| locked.map(|dictionary| dictionary.version.clone()));
    let cache_key = version
        .as_deref()
        .map_or_else(|| name.to_string(), |version| format!("{name}@{version}"));
    let path = crate::registry_cache_path(&cache_key)
        .context("cannot determine the registry cache directory")?;
    Ok(ResolvedRegistryRef {
        name: name.to_string(),
        version,
        sha256: locked.map(|dictionary| dictionary.sha256.clone()),
        path,
    })
}

pub fn verify(path: &Path, expected: &str) -> anyhow::Result<()> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("cannot read locked dictionary {}", path.display()))?;
    let actual = hex(&Sha256::digest(&bytes));
    anyhow::ensure!(
        actual == expected.to_ascii_lowercase(),
        "checksum mismatch for locked dictionary {} (expected {}, got {actual}); run `ayame-spell dict add` to restore it",
        path.display(),
        expected
    );
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lockfiles_are_sorted_and_dictionary_bytes_are_verified() {
        let directory = tempfile::tempdir().unwrap();
        let bytes = b"fixture\n";
        let digest = hex(&Sha256::digest(bytes));
        let mut lock = RegistryLock::default();
        lock.upsert(LockedDictionary {
            name: "z-last".to_string(),
            version: "1.0.0".to_string(),
            sha256: digest.clone(),
            file: "dicts/z-last.txt".to_string(),
        });
        lock.upsert(LockedDictionary {
            name: "a-first".to_string(),
            version: "1.2.0".to_string(),
            sha256: digest.clone(),
            file: "dicts/a-first.txt".to_string(),
        });
        let path = lock.save(directory.path()).unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        assert!(text.find("a-first").unwrap() < text.find("z-last").unwrap());
        let dictionary = directory.path().join("dictionary.txt");
        std::fs::write(&dictionary, bytes).unwrap();
        verify(&dictionary, &digest).unwrap();
        assert_eq!(split_reference("fixture@1.2.0"), ("fixture", Some("1.2.0")));
    }
}

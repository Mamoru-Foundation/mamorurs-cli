use serde::Deserialize;
use std::{collections::HashMap, path::Path};

#[derive(Debug, Deserialize, Clone)]
pub struct Manifest {
    pub name: String,
    pub version: HashMap<String, String>,
    pub subscribable: bool,
    pub description: String,

    #[serde(rename = "logoUrl")]
    pub logo_url: String,
    pub tags: Vec<String>,

    #[serde(rename = "chains")]
    pub supported_chains: Vec<String>,

    pub parameters: Option<Vec<ManifestParameter>>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct ManifestParameter {
    #[serde(rename = "type")]
    pub type_: String,
    pub title: String,
    pub key: String,
    pub description: String,
    #[serde(rename = "defaultValue")]
    pub default_value: String,
    #[serde(rename = "requiredFor")]
    pub required_for: Option<Vec<String>>,
    #[serde(rename = "hiddenFor")]
    pub hidden_for: Option<Vec<String>>,
    pub symbol: Option<String>,
    pub min: Option<String>,
    pub max: Option<String>,
    #[serde(rename = "minLen")]
    pub min_len: Option<u32>,
    #[serde(rename = "maxLen")]
    pub max_len: Option<u32>,
}

pub fn read_manifest_file(dir_path: &Path) -> Option<Manifest> {
    let manifest_path = dir_path.join("manifest.yaml");
    if !manifest_path.exists() {
        println!("Manifest file not found: {}", manifest_path.display());
        return None;
    }

    let manifest = serde_yaml::from_reader(std::fs::File::open(manifest_path).ok()?);
    match manifest {
        Ok(manifest) => Some(manifest),
        Err(e) => {
            println!("Error reading manifest file: {}", e);
            None
        }
    }
}

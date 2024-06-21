use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{any, fmt, io::Read};

pub fn get_deps(url: &str) -> Result<SpringResponse> {
    let response = reqwest::blocking::get(url)?;
    let response_json = response.json::<SpringResponse>()?;
    Ok(response_json)
}

pub fn get_zip(
    url: &str,
    dependencies: &str,
    build_type: &str,
    jvm: &str,
    artifact_id: &str,
    group_id: &str,
    language: &str,
    name: &str,
) -> Result<Vec<u8>> {
    let url = reqwest::Url::parse(url)?;
    let url = format!(
        "{}starter.zip?dependencies={}&type={}&javaVersion={}&artifactId={}&groupId={}&language={}&name={}",
        url,
        dependencies,
        build_type,
        jvm,
        artifact_id,
        group_id,
        language,
        name
    );

    let mut response = reqwest::blocking::get(url)?;
    if response.status() != 200 {
        anyhow::bail!("failed to get zip file status code: {}", response.status())
    }

    let mut buf: Vec<u8> = Vec::with_capacity(response.content_length().unwrap_or(0) as usize);
    let _ = response.read_to_end(&mut buf)?;
    Ok(buf)
}

impl fmt::Display for Dependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.id, self.name)
    }
}
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpringResponse {
    pub dependencies: Dependencies,
    pub java_version: JavaVersion,
    pub language: Language,
    pub group_id: GroupId,
    #[serde(rename = "type")]
    pub build_type: Types,
    pub artifact_id: ArtifactId,
    pub version: Version,
    pub name: Name,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependencies {
    #[serde(rename = "type")]
    pub type_field: String,
    pub values: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value {
    pub name: String,
    pub values: Vec<Dependency>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    pub id: String,
    pub name: String,
    pub description: String,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Types {
    #[serde(rename = "type")]
    pub type_field: String,
    pub default: String,
    pub values: Vec<Type>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Type {
    pub id: String,
    pub name: String,
    pub description: String,
    pub action: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    #[serde(rename = "type")]
    pub type_field: String,
    pub default: String,
    pub values: Vec<Jvm>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Jvm {
    pub id: String,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Language {
    #[serde(rename = "type")]
    pub type_field: String,
    pub default: String,
    pub values: Vec<Lang>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lang {
    pub id: String,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupId {
    #[serde(rename = "type")]
    pub type_field: String,
    pub default: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactId {
    #[serde(rename = "type")]
    pub type_field: String,
    pub default: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    #[serde(rename = "type")]
    pub type_field: String,
    pub default: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Name {
    #[serde(rename = "type")]
    pub type_field: String,
    pub default: String,
}

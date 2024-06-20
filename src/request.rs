use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fmt, io::Read};

pub fn get_deps() -> Result<SpringResponse> {
    let url: &str = "https://start.spring.io";
    let response = reqwest::blocking::get(url)?;
    let response_json = response.json::<SpringResponse>()?;
    Ok(response_json)
}

pub fn get_zip(
    dependencies: Vec<&Dependency>,
    build_type: &Type,
    jvm: String,
    artifact_id: String,
    group_id: String,
) -> Result<Vec<u8>> {
    let deps: Vec<String> = dependencies.iter().map(|d| d.id.clone()).collect();
    let url = format!(
        "https://start.spring.io/starter.zip?dependencies={}&type={}&javaVersion={}&artifactId={}&groupId={}",
        deps.join(","),
        build_type.id,
        jvm,
        artifact_id,
        group_id
    );
    let mut response = reqwest::blocking::get(url)?;
    let mut buf: Vec<u8> = Vec::with_capacity(response.content_length().unwrap_or(0) as usize);
    let _ = response.read_to_end(&mut buf)?;
    Ok(buf)
}

impl fmt::Display for Dependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} >> {}", self.id, self.name)
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
    pub group_id: GroupId,
    #[serde(rename = "type")]
    pub build_type: Types,
    pub artifact_id: ArtifactId,
    pub version: Version,
    pub name: Name,
    pub description: Description,
    pub package_name: PackageName,
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
    pub values: Vec<Java>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Java {
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Description {
    #[serde(rename = "type")]
    pub type_field: String,
    pub default: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageName {
    #[serde(rename = "type")]
    pub type_field: String,
    pub default: String,
}

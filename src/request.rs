use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fmt, io::Read};

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
    pub name: Name,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependencies {
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
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Types {
    pub default: String,
    pub values: Vec<Type>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Type {
    pub id: String,
    pub name: String,
    pub action: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
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
    pub default: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactId {
    pub default: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Name {
    pub default: String,
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

impl SpringResponse {
    pub fn get_options(url: &str) -> Result<Self> {
        let response = reqwest::blocking::get(url)?;
        if response.status() != 200 {
            anyhow::bail!(
                "failed to get options from {}, status code: {}",
                url,
                response.status()
            )
        }
        let response_json = response.json::<Self>()?;
        Ok(response_json)
    }
}

#[allow(clippy::too_many_arguments)]
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
        dependencies.trim(),
        build_type.trim(),
        jvm.trim(),
        artifact_id.trim(),
        group_id.trim(),
        language.trim(),
        name.trim()
    );

    let mut response = reqwest::blocking::get(url)?;
    if response.status() != 200 {
        anyhow::bail!("failed to get zip file status code: {}", response.status())
    }

    let content_length = response.content_length().unwrap_or(0);
    let mut buf: Vec<u8> = Vec::with_capacity(content_length as usize);
    let num = response.read_to_end(&mut buf)?;
    if num != content_length as usize {
        anyhow::bail!(
            "failed to read all bites, read {}, but got {} from server",
            num,
            content_length
        )
    }
    Ok(buf)
}

#[cfg(test)]
mod test {
    use super::*;
    use httpmock::prelude::*;

    #[test]
    fn test_get() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                        "dependencies": {"values": [{ "name": "Deps","values": [{"id": "native","name": "GraalVM Native Support"}]}]},
                        "type": {"default": "maven-project","values": [{"id": "maven-project","name": "Maven","action": "/starter.zip"}]},
                        "javaVersion": {
                            "default": "17",
                            "values": [{"id": "22","name": "22"},{"id": "21","name": "21"},{"id": "17","name": "17"}]},
                        "language": {
                            "default": "java",
                            "values": [{"id": "java","name": "Java"},{"id": "kotlin","name": "Kotlin"},{"id": "groovy","name": "Groovy"}]
                        },
                        "groupId": {"default": "com.example"},
                        "artifactId": {"default": "demo"},
                        "name": {"default": "demo"}
                    }"#,
                );
        });

        let res = SpringResponse::get_options(&server.url("/"));

        mock.assert();
        assert!(res.is_ok());
        let res = res.expect("is ok");
        assert_eq!(res.dependencies.values.len(), 1);
        assert_eq!(res.build_type.values.len(), 1);
        assert_eq!(res.language.values.len(), 3);
        assert_eq!(res.java_version.values.len(), 3);
        assert!(res.artifact_id.default.len() > 0);
        assert!(res.group_id.default.len() > 0);
        assert!(res.name.default.len() > 0);
    }

    #[test]
    fn test_get_zip() {
        let buf: Vec<u8> = vec![0, 0, 0, 0, 0, 8, 0, 0, 0];
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/starter.zip");
            then.status(200).body(&buf);
        });

        let res = get_zip(
            &server.url("/"),
            "",
            "maven",
            "22",
            "demo",
            "com.example",
            "java",
            "demo",
        );

        mock.assert();
        assert!(res.is_ok());
        let res = res.expect("is ok");
        assert_eq!(res, buf);
    }
}

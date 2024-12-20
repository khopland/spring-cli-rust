use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fmt, io::Read};
use urlencoding::encode;

use crate::steps::{ResponseStep, Step};

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

pub fn get_options(url: &str) -> Result<serde_json::Value> {
    println!("getting parameter from {}", &url);
    let response = reqwest::blocking::get(url)?;
    if response.status() != 200 {
        anyhow::bail!(
            "failed to get options from {}, status code: {}",
            url,
            response.status()
        )
    }
    let response_json = response.json()?;
    Ok(response_json)
}

#[allow(clippy::too_many_arguments)]
pub fn get_zip(url: &str, responses: &Vec<ResponseStep>) -> Result<(String, Vec<u8>)> {
    let mut url = reqwest::Url::parse(url)?;
    let path = get_path(&responses)
        .to_owned()
        .get_or_insert("starter.zip".to_string())
        .to_string();
    url.set_path(&path);
    let mut querys = url.query_pairs_mut();
    responses.iter().for_each(|q| {
        querys.append_pair(&encode(q.step.get_name()), &encode(&q.response));
    });
    drop(querys);

    let mut response = reqwest::blocking::get(url)?;
    if response.status() != 200 {
        anyhow::bail!("failed to get zip file status code: {}", response.status())
    }

    let content_length = response.content_length().unwrap_or(0);
    let content_file = &response
        .headers()
        .get("content-disposition")
        .map(|x| get_file_name(x))
        .flatten();

    let mut buf: Vec<u8> = Vec::with_capacity(content_length as usize);
    let num = response.read_to_end(&mut buf)?;
    if num != content_length as usize {
        anyhow::bail!(
            "failed to read all bites, read {}, but got {} from server",
            num,
            content_length
        )
    }
    Ok((
        content_file
            .to_owned()
            .get_or_insert("".to_owned())
            .to_string(),
        buf,
    ))
}

fn get_path(responses: &Vec<ResponseStep>) -> Option<String> {
    responses.iter().find_map(|r| match &r.step {
        Step::Action(s) => s
            .values
            .iter()
            .find(|x| x.id == r.response)
            .map(|x| x.action.clone()),
        _ => None,
    })
}

fn get_file_name(header: &reqwest::header::HeaderValue) -> Option<String> {
    let Ok(st) = header.to_str() else {
        return None;
    };

    let (_, s) = st.split_once("=")?;
    let mut chars = s.chars();
    chars.next();
    chars.next_back();
    Some(chars.as_str().to_owned())
}

#[cfg(test)]
mod test {
    use super::*;
    use httpmock::prelude::*;

    #[test]
    fn test_get_zip() {
        let buf: Vec<u8> = vec![0, 0, 0, 0, 0, 8, 0, 0, 0];
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/starter.zip");
            then.status(200).body(&buf);
        });

        let res = get_zip(&server.url("/"), &vec![]);

        mock.assert();
        assert!(res.is_ok());
        let res = res.expect("is ok");
        assert_eq!(res.1, buf);
    }
}

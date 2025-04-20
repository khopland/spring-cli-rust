use crate::steps::{ItemKind, Step, StepKind};
use anyhow::{Context, Result};
use std::str::FromStr;
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub struct ResponseStep {
    pub step: Step,
    pub response: String,
}

pub fn get_options(url: &str) -> Result<serde_json::Value> {
    let url = Url::from_str(&url)?.join("/metadata/config")?;

    println!("getting parameter from {}", &url);
    ureq::get(url.as_str())
        .call()?
        .body_mut()
        .read_json()
        .context("expect json back")
}

pub fn get_zip(url: &str, responses: &[ResponseStep]) -> Result<(Option<String>, Vec<u8>)> {
    let url = Url::parse(&url)?;

    let mut url = url.join(get_url_path(responses).context("action need to be set")?)?;
    let mut querys = url.query_pairs_mut();
    responses.iter().for_each(|q| {
        if !q.response.is_empty() {
            querys.append_pair(&q.step.name, &q.response.replace(" ", "%20"));
        }
    });
    drop(querys);

    let mut response = ureq::get(url.as_str()).call()?;

    let content_file = &response
        .headers()
        .get("content-disposition")
        .and_then(|header| {
            let Ok(value) = header.to_str() else {
                return None;
            };

            let (_, s) = value.split_once("=")?;
            let mut chars = s.chars();
            chars.next();
            chars.next_back();
            Some(chars.as_str().to_owned())
        });

    let body = response.body_mut();

    let buf: Vec<u8> = body.read_to_vec()?;

    Ok((content_file.to_owned().map(|x| format!("./{}", x)), buf))
}

fn get_url_path(responses: &[ResponseStep]) -> Option<&str> {
    responses.iter().find_map(|r| match &r.step.kind {
        StepKind::Action { values, .. } => {
            values
                .iter()
                .find(|x| x.id == r.response)
                .and_then(|step| match &step.kind {
                    ItemKind::Action(action) => Some(action.as_str()),
                    _ => None,
                })
        }
        _ => None,
    })
}

#[cfg(test)]
mod test {
    use crate::steps::{Item, Step};

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

        let res = get_zip(
            &server.url("/"),
            &vec![ResponseStep {
                step: Step {
                    name: "type".to_owned(),
                    kind: StepKind::Action {
                        default: "".to_string(),
                        values: vec![Item::new_action(
                            "java".to_string(),
                            "java".to_owned(),
                            "/starter.zip".to_owned(),
                        )],
                    },
                },
                response: "java".to_owned(),
            }],
        );

        mock.assert();
        assert!(res.is_ok());
        let res = res.expect("is ok");
        assert_eq!(res.1, buf);
    }
}

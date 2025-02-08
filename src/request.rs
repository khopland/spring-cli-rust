use crate::steps::{ItemKind, ResponseStep, StepKind};
use anyhow::Result;
use std::io::Read;

pub fn get_options(url: &str) -> Result<serde_json::Value> {
    println!("getting parameter from {}", &url);
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(url)
        .header("Accept", "application/vnd.initializr.v2.2+json")
        .send()?;

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

pub fn get_zip(url: &str, responses: &[ResponseStep]) -> Result<(Option<String>, Vec<u8>)> {
    let mut url = reqwest::Url::parse(url)?;
    let url_path = get_url_path(responses)
        .to_owned()
        .get_or_insert("starter.zip".to_string())
        .to_string();
    url.set_path(&url_path);

    let mut querys = url.query_pairs_mut();
    responses.iter().for_each(|q| {
        if !q.response.is_empty() {
            querys.append_pair(&q.step.name, &q.response.replace(" ", "%20"));
        }
    });
    drop(querys);

    let mut response = reqwest::blocking::get(url)?;
    if response.status() != 200 {
        anyhow::bail!(
            "failed to get zip file status code: {} - {}",
            response.status(),
            response.text()?
        )
    }

    let content_length = response.content_length().unwrap_or(0);
    let content_file = &response
        .headers()
        .get("content-disposition")
        .and_then(get_file_name);

    let mut buf: Vec<u8> = Vec::with_capacity(content_length as usize);
    let num = response.read_to_end(&mut buf)?;
    if num != content_length as usize {
        anyhow::bail!(
            "failed to read all bites, read {}, but got {} from server",
            num,
            content_length
        )
    }
    Ok((content_file.to_owned().map(|x| format!("./{}", x)), buf))
}

fn get_url_path(responses: &[ResponseStep]) -> Option<String> {
    responses.iter().find_map(|r| match &r.step.kind {
        StepKind::Action { values, .. } => {
            values
                .iter()
                .find(|x| x.id == r.response)
                .and_then(|step| match &step.kind {
                    ItemKind::Action(action) => Some(action.clone()),
                    _ => None,
                })
        }
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

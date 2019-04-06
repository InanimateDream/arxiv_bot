use super::*;
use reqwest::multipart::Form;

#[derive(Deserialize, Debug)]
pub struct Pin {
    pub id: String,
}

impl Default for Pin {
    fn default() -> Self {
        Self { id: "0".into() }
    }
}

impl From<&str> for Pin {
    fn from(id: &str) -> Self {
        Self { id: id.into() }
    }
}

impl TryFrom<Url> for Pin {
    type Error = Error;

    fn try_from(url: Url) -> Fallible<Self> {
        // TODO: same as arxiv_api, use lazy_static
        let regex = Regex::new(r"(?i)^(https://)?(www.)?zhihu.com/pin/(?P<id>\d+)(/)?$")?;
        let group = regex
            .captures(url.as_str())
            .ok_or_else(|| err_msg("Invalid url format."))?;
        Ok(Self {
            id: group
                .name("id")
                .unwrap()
                .as_str()
                .to_owned()
        })
    }
}

impl Pin {
    pub fn create(content: List, r#ref: Self, client: &Client) -> Fallible<Self> {
        // ensure the content list to be non-empty
        let content = if content.is_empty() {
            content.text("")
        } else {
            content
        };

        let form = Form::new()
            .text("content", serde_json::to_string(&content)?)
            .text("version", "1")
            .text("source_pin_id", r#ref.id);

        let mut resp = client
            .post("https://www.zhihu.com/api/v4/pins")
            .multipart(form)
            .send()?;

        check_status_code(&mut resp)?;

        // WORKAROUND:
        // The conversion from F: Fail to failure::Error only happens while throw an error by '?'
        Ok(resp.json()?)
    }

    pub fn delete(self, client: &Client) -> Fallible<()> {
        client
            .delete(&format!("https://www.zhihu.com/api/v4/pins/{}", self.id))
            .send()?;
        Ok(())
    }
}

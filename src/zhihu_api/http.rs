use super::*;
use reqwest::header::{
    COOKIE,
    HeaderMap,
    HeaderValue,
};

pub trait ClientExt {
    type Client;

    fn build(cookie: &str) -> Fallible<Self::Client>;
    fn is_valid(&self) -> Fallible<bool>; // cookie☆
}

impl ClientExt for Client {
    type Client = Self;

    fn build(cookie: &str) -> Fallible<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, HeaderValue::from_str(cookie)?);
        headers.insert("x-requested-with", HeaderValue::from_static("sakjdbasd"));

        // WORKAROUND:
        // The conversion from F: Fail to failure::Error only happens while throw an error by '?'
        Ok(Client::builder()
            .referer(false)
            .default_headers(headers)
            .build()?)
    }

    fn is_valid(&self) -> Fallible<bool> {
        let mut resp = self.get("https://www.zhihu.com/inbox").send()?;

        check_status_code(&mut resp)?;

        match resp.url().as_str() {
            "https://www.zhihu.com/inbox" => Ok(true),
            "https://www.zhihu.com/signup?next=%2Finbox" => Ok(false),
            _ => unreachable!(),
        }
    }
}

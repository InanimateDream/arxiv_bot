use super::*;
use reqwest::multipart::Form;
use serde::{
    ser::SerializeStruct,
    Serialize,
    Serializer
};

mod r#type {
    use super::*;

    #[derive(Debug, Deserialize)]
    pub struct Image {
        #[serde(rename = "src")]
        pub url: String,
        #[serde(rename = "data-rawwidth")]
        pub width: usize,
        #[serde(rename = "data-rawheight")]
        pub height: usize,
    }

    #[derive(Debug, Deserialize)]
    pub struct Link {
        pub url: String,
        pub title: String,
        pub image: String,
    }
}

#[derive(Debug)]
pub enum Content {
    Text(String),
    Image(r#type::Image),
    Link(r#type::Link),
}

impl Content {
    fn text(text: &str) -> Self {
        Content::Text(text.to_owned())
    }

    // TODO: use Path instead of &str
    fn image_path(path: &str, client: &Client) -> Fallible<Self> {
        let img = Form::new().file("picture", path)?;

        let mut resp = client
            .post("https://www.zhihu.com/api/v4/uploaded_images")
            .multipart(img)
            .send()?;

        check_status_code(&mut resp)?;

        Ok(Content::Image(resp.json()?))
    }

    fn image_url(path: &str, url: &str, client: &Client) -> Fallible<Self> {
        let mut img = reqwest::get(url)?;
        check_status_code(&mut img)?;

        let mut temp = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        img.copy_to(&mut temp)?;
        let ret = Self::image_path(path, client)?;

        fs::remove_file(path)?;
        Ok(ret)
    }

    fn link(url: &str, client: &Client) -> Fallible<Self> {
        let mut resp = client
            .get("https://www.zhihu.com/api/v3/scraper")
            .query(&[("url", url), ("image", "1")])
            .send()?;

        check_status_code(&mut resp)?;

        Ok(Content::Link(resp.json()?))
    }
}

impl Serialize for Content {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        match &self {
            Content::Text(s) => {
                let mut st = serializer.serialize_struct("Content", 2)?;
                st.serialize_field("type", "text")?;
                st.serialize_field("content", s)?;
                st.end()
            }
            Content::Image(r#type::Image { url, width, height }) => {
                let mut st = serializer.serialize_struct("Content", 4)?;
                st.serialize_field("type", "image")?;
                st.serialize_field("url", &url)?;
                st.serialize_field("width", &width)?;
                st.serialize_field("height", &height)?;
                st.end()
            }
            Content::Link(r#type::Link { url, title, image }) => {
                let mut st = serializer.serialize_struct("Content", 5)?;
                st.serialize_field("type", "link")?;
                st.serialize_field("isFetching", &false)?;
                st.serialize_field("title", &title)?;
                st.serialize_field("url", &url)?;
                st.serialize_field("imageUrl", &image)?;
                st.end()
            }
        }
    }
}

pub type List = Vec<Content>;

pub trait ContentList {
    type List;

    fn text(self, text: &str) -> Self::List;
    fn image_path(self, path: &str, client: &Client) -> Fallible<Self::List>;
    fn image_url(self, path: &str, url: &str, client: &Client) -> Fallible<Self::List>;
    fn link(self, url: &str, client: &Client) -> Fallible<Self::List>;
}

impl ContentList for List {
    type List = List;

    fn text(mut self, text: &str) -> List {
        self.push(Content::text(text));
        self
    }

    fn image_path(mut self, path: &str, client: &Client) -> Fallible<List> {
        self.push(Content::image_path(path, client)?);
        Ok(self)
    }

    fn image_url(mut self, path: &str, url: &str, client: &Client) -> Fallible<List> {
        self.push(Content::image_url(path, url, client)?);
        Ok(self)
    }

    fn link(mut self, url: &str, client: &Client) -> Fallible<List> {
        self.push(Content::link(url, client)?);
        Ok(self)
    }
}


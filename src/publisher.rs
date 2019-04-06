use super::prelude::*;

const AUTH_KEY: &str = "auth";

pub trait Publisher
    where Self: Sized
{
    type Auth;
    type Error;
    type Handle;

    fn new(auth: Self::Auth) -> Fallible<Self>;
    fn publish(&self, md: &MetaData, refs: Option<Self::Handle>) -> Result<Self::Handle, Self::Error>;
}

#[derive(Debug, Deserialize)]
pub struct Auth {
    cookie: String,
}

#[derive(Debug)]
pub struct Zhihu {
    client: Client
}

impl Publisher for Zhihu {
    type Auth = Auth;
    type Error = Error;
    type Handle = Pin;

    fn new(auth: Self::Auth) -> Fallible<Self> {
        Ok(Zhihu { client: Client::build(&auth.cookie)? })
    }

    fn publish(&self, md: &MetaData, refs: Option<Pin>) -> Fallible<Pin> {
        ensure!(self.client.is_valid()?, "Cookie out of date");

        let refs = refs.unwrap_or_else(Pin::default);
        Ok(Pin::create(format(&md, &self.client)?, refs, &self.client)?)
    }
}

// TODO: support mutli-backend publishing
pub fn init(env: &Env) -> Fallible<Zhihu> {
    let auth = toml::from_str(env
        .get(AUTH_KEY)
        .ok_or_else(|| err_msg("必须指定至少一种授权方式（Cookie或密码）"))?
        .to_string()
        .as_str())?;
    Zhihu::new(auth)
}

fn render(md: &MetaData) -> Fallible<Url> {
    let mut subs = md.sub
        .iter()
        .filter(|s| **s != md.prim_sub)
        .map(Subject::to_string)
        .collect::<Vec<_>>();
    if !subs.is_empty() {
        subs.insert(0, String::new());
    }

    let text = format!("$\
        \\textbf{{{}}}\
        \\vspace{{0.5em}}\
        \n\n\\textsc{{{}}}\
        \\vspace{{1em}}\
        \n\n{}\
        \\vspace{{1.5em}}\
        \n\nSubject Area(s): \\textbf{{{}}}{}\
        $", md.title
                       , md.auth.join(", ")
                       , md.abs
                       , md.prim_sub
                       , subs.join(", "))
        .replace(" ", "\n"); // Avoid stupid url encoding

    let client = Client::new();
    let mut resp = client.post("https://quicklatex.com/latex3.f")
        .form(&[
            ("formula", text.as_str()),
            ("fsize", "24px"),
            ("fcolor", "000000"),
            ("mode", "0"),
            ("out", "1"),
            ("preamble", "\\usepackage{amsmath}\n\
                \\usepackage{amsfonts}\n\
                \\usepackage{amssymb}\n\
                \\usepackage[a3paper]{geometry}\n\
                \\usepackage[mathletters]{ucs}\n\
                \\usepackage[utf8x]{inputenc}")
        ])
        .send()?;

    check_status_code(&mut resp)?;


    // TODO: use regex
    Ok(Url::parse(
        resp
            .text()?
            .split(' ')
            .nth(0)
            .ok_or_else(|| err_msg("LaTeX图片生成API格式错误"))?
            .split('\n')
            .nth(1)
            .ok_or_else(|| err_msg("LaTeX图片生成API格式错误"))?
    )?)
}

fn format(md: &MetaData, client: &Client) -> Fallible<List> {
    let authors = if md.auth.len() > 2 {
        format!("{} et. al.", md.auth[0])
    } else {
        md.auth.join(", ")
    };

    let url: Url = (&md.index).into();
    // TODO: configurable temporarily image path
    List::new()
        .text(&format!("<p>{}: {}</p>", authors, md.title))
        .link(url.as_str(), client)?
        .image_url("/tmp/temp.pngg", render(md)?.as_str(), client)
}

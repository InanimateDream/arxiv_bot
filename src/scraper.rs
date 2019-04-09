use super::prelude::*;

const SCRAPE_KEY: &str = "scraper";

// TODO: Use &str to avoid clone
#[derive(Debug, Deserialize)]
pub struct Scraper {
    source: Option<String>,
    subject: Vec<String>,
}

impl Scraper {
    pub fn scrape(&self, conn: SqlConn) -> Fallible<Vec<Rss>> {
        let src = self.source
            .clone()
            .unwrap_or_else(|| "rss".to_owned());

        match src.as_str() {
            "rss" => {
                self.subject
                    .iter()
                    .map(|s| Subject::from_str(s.as_str()))
                    .try_fold(Vec::new(), try_fold_helper)?
                    .into_iter()
                    .map(Rss::try_from)
                    .try_fold(Vec::new(), |mut acc, rss| {
                        if let Ok(rss) = rss {
                            if rss.last() > paper::last(conn, &rss.sub)?
                                .unwrap_or(DateTime::parse_from_rfc3339("1970-01-01T00:00:00-00:00")?) {
                                acc.push(rss);
                            } else {
                                info!("本次没有更新，等待下一次轮询");
                            }
                            Ok(acc)
                        } else {
                            Err(rss.unwrap_err())
                        }
                    })
            },
            _ => bail!("无效的抓取源：{}", src)
        }
    }
}

pub fn init(env: &Env) -> Fallible<Scraper> {
    Ok(toml::from_str(env
        .get(SCRAPE_KEY)
        .ok_or_else(|| err_msg("至少需要指定一个订阅的学科"))?
        .to_string()
        .as_str())?)
}
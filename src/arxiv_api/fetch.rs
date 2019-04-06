use super::*;
use rss::Channel;

// We discards all metadata contained in the list pages since none of them
// contains all the abstracts, subjects and authors completely.
pub trait PaperList {
    fn last(&self) -> DateTime<FixedOffset>;

    fn link(&self) -> Vec<Url>;
    fn index(&self) -> Vec<Index>;
}

// TODO: Add /current /new /recent and specific dates

#[derive(Debug)]
pub struct Rss{
    pub sub: Subject,
    feed: Channel,
}

impl TryFrom<Subject> for Rss {
    type Error = failure::Error;
    fn try_from(sub: Subject) -> Fallible<Self> {
        // TODO: Parse subs
        let feed = Channel::from_url(&format!("http://export.arxiv.org/rss/{}?version=2.0", sub.as_str()))?;
        Ok(Rss{ sub, feed })
    }
}

// In this trait we regard the arXiv RSS feed as a stable interface and assume the inputs are
// always valid so every Results and Options are directly unwrapped
impl PaperList for Rss {
    fn last(&self) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc2822(self.feed.last_build_date().unwrap()).unwrap()
    }

    fn link(&self) -> Vec<Url> {
        self.feed
            .items()
            .iter()
            .map(|item| {
                // If the whole rss feed is valid, the link must be valid
                // Moreover, handle errors within a closure would mess everything up
                Url::parse(item.link().unwrap()).unwrap()
            })
            .collect()
    }

    fn index(&self) -> Vec<Index> {
        self.feed
            .items()
            .iter()
            .map(|item| {
                // Same reason, if arXiv itself produces a incorrect paper index, let it crash
                item.guid()
                    .unwrap()
                    .value()
                    .split(':') // oai:arXiv.org:{index}
                    .nth(2)
                    .unwrap()
                    .parse::<Index>()
                    .unwrap()
            }).collect()
    }
}

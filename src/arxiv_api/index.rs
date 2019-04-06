use super::*;
use chrono::NaiveDate as Date;

// Before 0704: arch-ive/YYMMNNN
// Now: YYMM.NNNN or YYMM.NNNNN
// We don't care about the actual day of a Date object so we always let it be the 1st
#[derive(Clone, Debug)]
pub enum Index {
    Old { date: Date, idx: usize, sub: Subject },
    New { date: Date, idx: usize }
}

impl FromStr for Index {
    type Err = Error;
    fn from_str(id: &str) -> Fallible<Self> {
        // old: subject(.sub-subject)?/YYMMNNN
        // new: YYMM.(N)NNNN

        // safely unwrap for constant
        // TODO: Use lazy-static to prevent multiple compilation
        let old = Regex::new(
            r"(?i)^(?P<sub>[a-z\-]+)(\.(?P<ssub>[a-z\-]+))?/(?P<dt>\d{4})(?P<ix>\d{3})$"
        ).unwrap();
        let new = Regex::new(r"^(?P<dt>\d{4})\.(?P<ix>\d{4,5})$").unwrap();

        // All safely unwrapped, since the string is fully matched to a constant regex
        // This is the most common path in my case
        if new.is_match(id) {
            let group = new.captures(id).unwrap();

            let date = Date::parse_from_str(
                &format!("{}01", group.name("dt").unwrap().as_str()),
                "%y%m%d"
            )?;
            let idx = group.name("ix").unwrap().as_str().parse::<usize>()?;

            Ok(Index::New { date, idx })
        }
        // and this is extremely rare, so we use a simple assertion
        // to bypass an relative expensive regex match
        else if id.contains('/') && old.is_match(id) {
            let group = old.captures(id).unwrap();

            // TODO: Parse the Subject and sub-subject
            let sub = Subject::from_str(group.name("sub").unwrap().as_str())?;
            let date = Date::parse_from_str(
                &format!("{}01", group.name("dt").unwrap().as_str()),
                "%y%m%d"
            )?;
            let idx = group.name("ix").unwrap().as_str().parse::<usize>()?;

            Ok(Index::Old { date, idx, sub })

        }
        // this is even more common than the second branch
        else {
            bail!("Invalid Index: {}, expect format: YYMM.NNNNN or arch-ive/YYMMNNN", id);
        }
    }
}

// All Results are safely unwrapped, the inputs are totally predictable
// because every part of the url are formatted by the current module
impl Into<Url> for &Index {
    fn into(self) -> Url {
        let path = Url::parse("https://arxiv.org/abs/").unwrap();
        match self {
            // TODO: formatting the subject
            // If there's no slash, it'll be dropped when join another location on it
            Index::Old { ref sub, .. } => path.join(&format!("{}/", sub.as_str())).unwrap(),
            Index::New {..} => path,
        }
            .join(&self.to_string())
            .unwrap()
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Index::Old { date, idx, .. } =>
                write!(f, "{}{:03}", date.format("%y%m").to_string(), idx),
            Index::New { date, idx } =>
                write!(f, "{}.{:05}", date.format("%y%m").to_string(), idx),
        }
    }
}

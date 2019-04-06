use super::schema::{
    pins as p,
    papers as pr,
    authors as at,
    subjects as sb,
    update_time as ut,
};
use super::prelude::*;
use diesel::prelude::*;
use diesel::dsl::*;

const DB_KEY: &str = "db";

pub type SqlConn<'a> = &'a SqliteConnection;
type MDList = Vec<MetaData>;

#[derive(Deserialize, Debug)]
struct DbConfig {
    database_path: String,
}

#[derive(Queryable, Insertable, Debug)]
#[table_name = "pr"]
struct Paper {
    arxiv_id: String,
    title: String,
    abstract_: String,
    prim_sub: String,
}

impl From<MetaData> for Paper
{
    fn from(md: MetaData) -> Self {
        Paper {
            arxiv_id: md.index.to_string(),
            title: md.title,
            abstract_: md.abs,
            prim_sub: md.prim_sub.as_str().to_owned(),
        }
    }
}

impl<'a> TryFrom<(SqlConn<'a>, Paper)> for MetaData {
    type Error = Error;
    fn try_from((conn, p): (SqlConn, Paper)) -> Fallible<Self> {
        use at::dsl::{
            *,
            arxiv_id as aid
        };
        use sb::dsl::{
            *,
            arxiv_id as sid
        };

        Ok(MetaData {
            title: p.title,
            abs: p.abstract_,
            prim_sub: Subject::from_str(&p.prim_sub)?,
            index: Index::from_str(&p.arxiv_id)?,
            sub: subjects
                .select(sub)
                .filter(sid.eq(p.arxiv_id.clone()))
                .load::<String>(conn)?
                .into_iter()
                .map(|s| Subject::from_str(&s))
                .try_fold(Vec::new(), try_fold_helper)?,
            auth: authors
                .select(auth)
                .filter(aid.eq(p.arxiv_id))
                .load::<String>(conn)?,
        })
    }
}

pub fn init(env: &Env) -> Fallible<SqliteConnection> {
    let dbc: DbConfig = toml::from_str(env
        .get(DB_KEY)
        .ok_or_else(|| err_msg("必须指定一个数据库路径"))?
        .to_string()
        .as_str())?;

    let conn = SqliteConnection::establish(dbc.database_path.as_str())?;
    conn.execute("PRAGMA foreign_keys = ON;")?;
    Ok(conn)
}

pub mod paper {
    use super::*;

    pub fn insert(conn: SqlConn, subj: Subject, md: MetaData, tm: DateTime<FixedOffset>) -> Fallible<()> {
        use ut::dsl::*;
        use pr::dsl::papers;
        use at::dsl::{
            auth,
            authors,
            arxiv_id as aid,
        };
        use sb::dsl::{
            sub,
            subjects,
            arxiv_id as sid,
        };

        // Need partially moved data types
        insert_into(papers)
            .values(Paper::from(md.clone()))
            .execute(conn)?;

        for at in md.auth {
            insert_into(authors)
                .values((aid.eq(md.index.to_string()), auth.eq(at)))
                .execute(conn)?;
        }

        for sb in md.sub {
            insert_into(subjects)
                .values((sid.eq(md.index.to_string()), sub.eq(sb.as_str())))
                .execute(conn)?;
        }

        if tm > last(conn, &subj)? {
            insert_into(update_time)
                .values((subject.eq(subj.as_str()), rss_time.eq(tm.to_rfc3339())))
                .execute(conn)?;
        }

        Ok(())
    }

    pub fn by_id(conn: SqlConn, idx: Index) -> Fallible<MetaData> {
        use pr::dsl::*;
        let p = papers
            .find(idx.to_string())
            .first::<Paper>(conn)?;
        MetaData::try_from((conn, p))
    }

    pub fn by_subs(conn: SqlConn, subs: Vec<Subject>) -> Fallible<MDList> {
        use pr::dsl::*;
        use sb::dsl::{
            sub,
            subjects,
        };

        papers
            .inner_join(subjects)
            .filter(sub.eq_any(
                subs.iter()
                    .map(Subject::as_str)
                    .collect::<Vec<_>>()
            ))
            .select(arxiv_id)
            .distinct()
            .load::<String>(conn)?
            .into_iter()
            .map(|ix| {
                by_id(conn, Index::from_str(&ix)?)
            })
            .try_fold(Vec::new(), try_fold_helper)
    }

    pub fn by_prim_sub(conn: SqlConn, sub: Subject) -> Fallible<MDList> {
        use pr::dsl::*;

        papers
            .filter(prim_sub.eq(sub.as_str()))
            .load::<Paper>(conn)?
            .into_iter()
            .map(|p| MetaData::try_from((conn, p)))
            .try_fold(Vec::new(), try_fold_helper)
    }

    pub fn by_auths(conn: SqlConn, auths: Vec<String>) -> Fallible<MDList> {
        use pr::dsl::*;
        use at::dsl::{
            auth,
            authors,
        };

        papers
            .inner_join(authors)
            .filter(auth.eq_any(auths))
            .select(arxiv_id)
            .distinct()
            .load::<String>(conn)?
            .into_iter()
            .map(|ix| {
                by_id(conn, Index::from_str(&ix)?)
            })
            .try_fold(Vec::new(), try_fold_helper)
    }

    pub fn last(conn: SqlConn, sub: &Subject) -> Fallible<DateTime<FixedOffset>> {
        use ut::dsl::*;

        Ok(update_time
            .select(rss_time)
            .filter(subject.eq(sub.as_str()))
            .distinct()
            .load::<String>(conn)?
            .into_iter()
            .map(|tm| {
                DateTime::parse_from_rfc3339(&tm)
            })
            .try_fold(DateTime::parse_from_rfc3339("1970-01-01T00:00:00-00:00")?, |acc, tm| {
                if let Ok(tm) = tm {
                    if acc > tm { Ok(acc) } else { Ok(tm) }
                } else {
                    Err(tm.unwrap_err()) // safely unwrap
                }
            })?)
    }
}

pub mod pin {
    use super::*;

    pub fn insert(conn: SqlConn, pin: Pin, r#ref: Option<Pin>, idx: Index) -> Fallible<()> {
        use p::dsl::*;

        if let Some(rid) = r#ref {
            insert_into(pins)
                .values((
                    id.eq(pin.id),
                    ref_id.eq(rid.id),
                    arxiv_id.eq(idx.to_string())
                ))
                .execute(conn)?;
        } else {
            insert_into(pins)
                .values((
                    id.eq(pin.id),
                    arxiv_id.eq(idx.to_string())
                ))
                .execute(conn)?;
        }

        Ok(())
    }

    pub fn by_arxiv_id(conn: SqlConn, idx: Index) -> Fallible<Pin> {
        use p::dsl::*;

        Ok(Pin::from(pins
            .select(id)
            .filter(arxiv_id.eq(idx.to_string()))
            .first::<String>(conn)?
            .as_str()))
    }
}

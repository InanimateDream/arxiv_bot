use super::*;
use select::{
    document::Document,
    predicate::{Or, Child, Class, Name},
};

#[derive(Clone, Debug)]
pub struct MetaData {
    pub index: Index,
    pub title: String,
    pub auth: Vec<String>,
    pub abs: String,
    pub sub: Vec<Subject>,
    pub prim_sub: Subject,
}

// This method might be changed frequently, depends on any slight
// change on the layout of the arXiv web pages.
// We need a stable comprehensive JSON API, not RSS feeds.
impl TryFrom<Index> for MetaData {
    type Error = failure::Error;
    fn try_from(index: Index) -> Fallible<Self> {
        let e = "Scraping arXiv metadata failed, this might \
        be caused by changes on the display format of the arXiv \
        web pages, please contact the maintainer.";

        let url: Url = (&index).into();
        let mut resp = reqwest::get(url)?;

        check_status_code(&mut resp)?;

        let document = Document::from_read(resp)?;

        // <h1 class="title mathjax">
        //   <span class="descriptor">
        //     Title:
        //   </span>
        //   {title}
        // </h1>
        let title = document
            .find(Class("title"))
            .nth(0)
            .ok_or_else(|| err_msg(e))?
            .last_child()
            .ok_or_else(|| err_msg(e))?
            .text();

        // <div class="authors">
        //   <span class="descriptor">
        //     Authors:
        //   </span>
        //   <a href="{link_1}">
        //     {author_1}
        //   </a>
        //   ,
        //   <a href="{link_2}">
        //     {author_2}
        //   </a>
        //   ,
        //   ...
        //   <a href="{link_n}">
        //     {author_n}
        //   </a>
        // </div>
        let auth = document
            .find(Child(Class("authors"), Name("a")))
            .map(|n| n.text())
            .collect();

        // <blockquote class="abstract mathjax">
        //   <span class="descriptor">
        //     Abstract:
        //   </span>
        //     {abs}
        // </blockquote>
        //
        // * Note that there's two whitespace before the abstract
        let abs = document
            .find(Class("abstract"))
            .nth(0)
            .ok_or_else(|| err_msg(e))?
            .children()
            .filter_map(|node| {
                if node.is(Or(Class("descriptor"), Name("br"))) {
                    None
                } else {
                    Some(node.text().trim().to_owned())
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        // <td class="tablecell subjects">
        //   <span class="primary-subject">
        //     {sub_1}
        //   </span>
        //   ; {sub_2}...; {sub_n}
        // </td>
        let sub = document
            .find(Class("subjects"))
            .nth(0)
            .ok_or_else(|| err_msg(e))?
            .text()
            .split("; ")
            .map(Subject::from_str)
            .try_fold(Vec::new(), try_fold_helper)?;

        let prim_sub = Subject::from_str(document
            .find(Class("primary-subject"))
            .nth(0)
            .ok_or_else(|| err_msg(e))?
            .text()
            .as_str())?;

        Ok(MetaData { index, title, auth, abs, sub, prim_sub })
    }
}

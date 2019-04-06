#![feature(try_from)]

use arxiv_bot::prelude::*;
use std::{fs, env, process};

fn main() -> Fallible<()> {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "arxiv_bot.toml".to_owned());

    fs::read_to_string(path.as_str())
        .map_err(Error::from)
        .and_then(|config| {
            config.parse::<Env>()
                .map_err(Error::from)
        })
        .map_err(|e| {
            eprintln!("{}: {}", path, e);
            process::exit(1)
        })
        .and_then(launch)
        .map_err(handle)
}

fn launch(env: Env) -> Fallible<()> {
    init::log(&env)?;

    let conn = init::db(&env)?;
    let scraper = init::scraper(&env)?;
    let publisher = init::pub_(&env)?;

    let timer = init::timer(&env, move |t| {
        loop {
            let worker = || -> Fallible<()> {
                for rss in scraper.scrape(&conn)? {
                    for ix in rss.index() {
                        let md = MetaData::try_from(ix)?;
                        paper::insert(&conn, rss.sub.clone(), md.clone(), rss.last())?;
                        let pin = publisher.publish(&md, None)?;
                        pin::insert(&conn, pin, None, md.index.clone())?;
                    }
                }
                Ok(())
            };

            if let Err(e) = worker() {
                error!("检测到错误，本次运行失败，下一次运行将在{}秒后.", t.as_secs());
                error!("错误详情：{}", e);
            }

            sleep(t);
        }
    })?;

    timer.join().unwrap() // unreachable, safely unwrap
}

fn handle(e: Error) -> Error {
    // TODO: HANDLE ERRORS
    error!("{}", e);
    e
}

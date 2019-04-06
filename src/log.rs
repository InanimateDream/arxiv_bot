use super::prelude::*;
use simplelog::*;

#[derive(Debug, Deserialize)]
struct Console { level: String }
#[derive(Debug, Deserialize)]
struct File { path: String, level: String }

impl File {
    fn parse(self) -> Fallible<(fs::File, LevelFilter)> {
        let level = self.level.parse()?;
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(self.path)?;

        Ok((file, level))
    }
}

#[derive(Debug, Deserialize)]
struct Log {
    file: Option<Vec<File>>,
    console: Option<Console>,
}

impl Default for Log {
    fn default() -> Self {
        let f = vec![ File {
            path: "arxiv_bot.log".to_owned(),
            level: "warn".to_owned()
        } ];

        Self {
            file: Some(f),
            console: None
        }
    }
}

const LOG_KEY: &str = "log";

pub fn init(env: &Env) -> Fallible<()> {
    let conf = env
        .get(LOG_KEY)
        .map_or_else(|| Ok(Log::default()), |v| toml::from_str(v.to_string().as_str()))?;

    let conf = if conf.file.is_none() && conf.console.is_none() {
        eprintln!("检测到`{}`项存在但配置不正确，载入默认日志配置", LOG_KEY);
        Log::default()
    } else { conf };

    match conf {
        Log { file: Some(mut f), console: None } => {
            if f.len() == 1 {
                let (f, l) = f.pop().unwrap().parse()?;
                WriteLogger::init(l, Config::default(), f)?;
            } else {
                let mut logger: Vec<Box<dyn SharedLogger>> = Vec::new();
                for f in f {
                    let (f, l) = f.parse()?;
                    logger.push(WriteLogger::new(l, Config::default(), f));
                }
                CombinedLogger::init(logger)?;
            }
        },
        Log { file: None, console: Some(c) } => {
            TermLogger::init(c.level.parse()?, Config::default())?;
        },
        Log { file: Some(f), console: Some(c) } => {
            let mut logger: Vec<Box<dyn SharedLogger>> = Vec::new();
            logger.push(TermLogger::new(c.level.parse()?, Config::default()).unwrap());
            for f in f {
                let (f, l) = f.parse()?;
                logger.push(WriteLogger::new(l, Config::default(), f));
            }
            CombinedLogger::init(logger)?;
        },
        _ => unreachable!()
    };

    Ok(())
}

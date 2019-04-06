use super::prelude::*;

const TIMER_KEY: &str = "timer";

#[derive(Debug, Deserialize)]
struct Timer {
    interval: toml::value::Datetime
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            interval: "08:00:00".parse().unwrap() // safely unwrap
        }
    }
}

pub fn init<F, T>(env: &Env, f: F) -> Fallible<JoinHandle<T>> where
    F: FnOnce(std::time::Duration) -> T,
    F: Send + 'static,
    T: Send + 'static
{
    let itv = env
        .get(TIMER_KEY)
        .map_or_else(|| Ok(Timer::default()), |v| toml::from_str(v.to_string().as_str()))?;

    let dur = NaiveTime::parse_from_str(
        itv.interval
            .to_string()
            .as_str(), "%H:%M:%S"
    )?
        .signed_duration_since(NaiveTime::from_hms(0, 0, 0))
        .to_std()?;

    let tm = spawn(move || f(dur));
    Ok(tm)
}

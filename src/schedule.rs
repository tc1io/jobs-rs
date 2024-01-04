use cron::Schedule;
use std::str::FromStr;

pub fn secondly() -> Schedule {
    Schedule::from_str("* * * * * *").expect("secondly cron expression should parse")
}
pub fn minutely() -> Schedule {
    Schedule::from_str("0 * * * * *").expect("minutely cron expression should parse")
}
pub fn everyFiveMinutes() -> Schedule {
    Schedule::from_str("0 */5 * * * *").expect("everyFixeMinutes cron expression should parse")
}

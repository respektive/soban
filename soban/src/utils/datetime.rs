use time::OffsetDateTime;

// https://github.com/MaxOhn/Bathbot/blob/main/bathbot-util/src/datetime.rs#L28-L89

pub trait RelativeTime {
    fn to_relative(&self) -> String;
}

impl RelativeTime for OffsetDateTime {
    fn to_relative(&self) -> String {
        let now = OffsetDateTime::now_utc();
        let diff_sec = now.unix_timestamp() - self.unix_timestamp();
        debug_assert!(diff_sec >= 0);

        let one_day = 24 * 3600;
        let one_week = 7 * one_day;

        let (amount, unit) = {
            if diff_sec < 60 {
                (diff_sec, "second")
            } else if diff_sec < 3600 {
                (diff_sec / 60, "minute")
            } else if diff_sec < one_day {
                (diff_sec / 3600, "hour")
            } else if diff_sec < one_week {
                (diff_sec / one_day, "day")
            } else if diff_sec < 4 * one_week {
                (diff_sec / one_week, "week")
            } else {
                let diff_month = (12 * (now.year() - self.date().year()) as u32
                    + now.month() as u32
                    - self.date().month() as u32) as i64;

                if diff_month < 1 {
                    (diff_sec / one_week, "week")
                } else if diff_month < 12 {
                    (diff_month, "month")
                } else {
                    let years = diff_month / 12 + (diff_month % 12 > 9) as i64;

                    (years, "year")
                }
            }
        };

        format!(
            "{amount} {unit}{plural} ago",
            plural = if amount == 1 { "" } else { "s" }
        )
    }
}

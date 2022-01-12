use anyhow::Context;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

fn consume_and_pad(s: &mut &str, n: usize) -> String {
    let (consumed, new_s) = s.split_at(n.min(s.len()));
    *s = new_s;
    format!("{:0<width$}", consumed, width = n)
}
pub fn parse_unformatted_datetime(datetime: &str) -> anyhow::Result<NaiveDateTime> {
    let datetime = datetime
        .chars()
        .filter(|c| c.is_numeric())
        .collect::<String>();
    let mut datetime = datetime.as_str();

    let year = consume_and_pad(&mut datetime, 4).parse::<i32>()?;
    let month = consume_and_pad(&mut datetime, 2).parse::<u32>()?.max(1);
    let day = consume_and_pad(&mut datetime, 2).parse::<u32>()?.max(1);
    let hour = consume_and_pad(&mut datetime, 2).parse::<u32>()?;
    let minute = consume_and_pad(&mut datetime, 2).parse::<u32>()?;
    let second = consume_and_pad(&mut datetime, 2).parse::<u32>()?;
    Ok(NaiveDateTime::new(
        NaiveDate::from_ymd_opt(year, month, day)
            .with_context(|| format!("Invalid YMD {}-{}-{}", year, month, day))?,
        NaiveTime::from_hms_opt(hour, minute, second)
            .with_context(|| format!("Invalid HMS: {}:{}:{}", hour, minute, second))?,
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    fn naive_date(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> NaiveDateTime {
        NaiveDateTime::new(
            chrono::NaiveDate::from_ymd(year, month, day),
            chrono::NaiveTime::from_hms(hour, minute, second),
        )
    }
    #[test]
    fn test_parse_unformatted_datetime() {
        assert_eq!(
            parse_unformatted_datetime("200").unwrap(),
            naive_date(2000, 1, 1, 0, 0, 0)
        );
        assert_eq!(
            parse_unformatted_datetime("2022-02").unwrap(),
            naive_date(2022, 2, 1, 0, 0, 0)
        );
        assert_eq!(
            parse_unformatted_datetime("2022-03-30 16").unwrap(),
            naive_date(2022, 3, 30, 16, 0, 0)
        );
        assert_eq!(
            parse_unformatted_datetime("2022-03-30 16:30:5").unwrap(),
            naive_date(2022, 3, 30, 16, 30, 50)
        );
        assert_eq!(
            parse_unformatted_datetime("2022-03-30 16:30:51").unwrap(),
            naive_date(2022, 3, 30, 16, 30, 51)
        );

        // Invalid dates
        assert_eq!(
            parse_unformatted_datetime("2022-2")
                .unwrap_err()
                .to_string(),
            "Invalid YMD 2022-20-1"
        );
    }
}

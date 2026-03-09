use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecurrenceRule {
    pub frequency: Frequency,
    pub interval: i32,
    pub days_of_week: Option<Vec<DayOfWeek>>,
    pub day_of_month: Option<i32>,
    pub end_date: Option<NaiveDate>,
}

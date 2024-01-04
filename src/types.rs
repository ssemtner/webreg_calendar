use chrono::{NaiveDate, NaiveTime};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Session {
    pub section: Option<String>,
    pub session_type: SessionType,
    pub days: Days,
    pub timeslot: Option<Timeslot>,
    pub building: String,
    pub room: String,
}

#[derive(Debug)]
pub struct Course {
    pub code: String,
    pub title: String,
    pub sessions: Vec<Session>,
    pub instructor: String,
    pub units: u8,
    pub uuid: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Debug, Clone)]
pub enum SessionType {
    Lecture,
    Discussion,
    Final,
    Lab,
    Midterm,
    Tutorial,
    Seminar,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturady,
    Sunday,
}

impl Day {
    pub fn calendar_string(&self) -> &str {
        match self {
            Day::Monday => "MO",
            Day::Tuesday => "TU",
            Day::Wednesday => "WE",
            Day::Thursday => "TH",
            Day::Friday => "FR",
            Day::Saturady => "SA",
            Day::Sunday => "SU",
        }
    }
}

impl Into<chrono::Weekday> for &Day {
    fn into(self) -> chrono::Weekday {
        match self {
            Day::Monday => chrono::Weekday::Mon,
            Day::Tuesday => chrono::Weekday::Tue,
            Day::Wednesday => chrono::Weekday::Wed,
            Day::Thursday => chrono::Weekday::Thu,
            Day::Friday => chrono::Weekday::Fri,
            Day::Saturady => chrono::Weekday::Sat,
            Day::Sunday => chrono::Weekday::Sun,
        }
    }
}

impl From<&chrono::Weekday> for Day {
    fn from(value: &chrono::Weekday) -> Self {
        match value {
            chrono::Weekday::Mon => Day::Monday,
            chrono::Weekday::Tue => Day::Tuesday,
            chrono::Weekday::Wed => Day::Wednesday,
            chrono::Weekday::Thu => Day::Thursday,
            chrono::Weekday::Fri => Day::Friday,
            chrono::Weekday::Sat => Day::Saturady,
            chrono::Weekday::Sun => Day::Sunday,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Days {
    Date(NaiveDate),
    Days(Vec<Day>),
}

#[derive(Debug, Clone)]
pub struct Timeslot {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

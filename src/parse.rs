use anyhow::{anyhow, Result};
use chrono::{NaiveDate, NaiveTime};
use scraper::{Html, Selector};
use uuid::Uuid;

use crate::types::{Course, Day, Days, Session, SessionType, Timeslot};

impl Course {
    pub fn from_html(
        html: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<Course>> {
        let document = Html::parse_document(html);
        let row_selector = Selector::parse("#list-id-table > tbody > tr").unwrap();
        let rows = document.select(&row_selector);
        let col_selector = Selector::parse("td").unwrap();

        let rows = rows
            .skip(1)
            .filter_map(|row| {
                let cols = row
                    .select(&col_selector)
                    .map(|col| col.inner_html())
                    .collect();
                Row::parse(cols)
            })
            .collect::<Vec<_>>();

        Course::from_rows(rows, start_date, end_date)
    }

    fn from_rows(
        rows: Vec<Row>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<Course>> {
        let mut courses = Vec::new();
        let mut sessions = Vec::new();

        for row in rows.iter().rev() {
            // Add current row to sessions
            sessions.push(Session::from_row(row)?);

            if let Some(course_code) = &row.code {
                // End previous course
                courses.push(Course {
                    code: course_code.clone(),
                    title: row.title.clone().ok_or(anyhow!("Course has no title"))?,
                    sessions: sessions.clone(),
                    instructor: row
                        .instructor
                        .clone()
                        .ok_or(anyhow!("Course has no instructor"))?,
                    units: row.units.clone().ok_or(anyhow!("Course has no units"))?,
                    uuid: Uuid::new_v4(),
                    start_date,
                    end_date,
                });
                // Start new course
                sessions.clear();
            }
        }

        courses.reverse();

        Ok(courses)
    }
}

impl Session {
    fn from_row(row: &Row) -> Result<Session> {
        Ok(Self {
            section: row.section.clone(),
            session_type: row
                .session_type
                .clone()
                .ok_or(anyhow!("Row has no section"))?,
            days: row.days.clone().ok_or(anyhow!("Row has no days"))?,
            timeslot: row.timeslot.clone(),
            building: row.building.clone().ok_or(anyhow!("Row has no building"))?,
            room: row.room.clone().ok_or(anyhow!("Row has no room"))?,
        })
    }
}

#[derive(Debug)]
pub struct Row {
    code: Option<String>,
    title: Option<String>,
    section: Option<String>,
    session_type: Option<SessionType>,
    instructor: Option<String>,
    units: Option<u8>,
    days: Option<Days>,
    timeslot: Option<Timeslot>,
    building: Option<String>,
    room: Option<String>,
}

impl Row {
    fn parse(cols: Vec<String>) -> Option<Row> {
        fn remove_repeated_spaces(str: &str) -> String {
            let mut new_str = str.trim().to_owned();
            let mut prev = ' ';
            new_str.retain(|c| {
                let result = c != ' ' || prev != ' ';
                prev = c;
                result
            });
            new_str
        }

        fn optional(str: &str) -> Option<String> {
            match str.trim() {
                "" => None,
                str => Some(remove_repeated_spaces(str).replace("&amp;", "&")),
            }
        }

        // Returns the first string between '>' and '<' in the input
        // Ignores "TBA" which is not in a link
        fn remove_tag(html: &str) -> Option<String> {
            match html {
                "" => None,
                "TBA" => Some(String::from("TBA")),
                html => Some(
                    html.chars()
                        .skip_while(|&c| c != '>')
                        .skip(1)
                        .take_while(|&c| c != '<')
                        .collect::<String>(),
                ),
            }
        }

        // Rows that are expand buttons can be ignored, the expanded content is already included
        if cols.iter().any(|col| col.contains("Expand:")) {
            return None;
        }

        Some(Row {
            code: optional(&cols[0]),
            title: optional(&cols[1]),
            section: optional(&cols[2]),
            session_type: SessionType::parse(&cols[3]),
            instructor: remove_tag(&cols[4]),
            units: cols[6].trim().parse::<f32>().ok().map(|x| x as u8),
            days: Days::parse(&cols[7]),
            timeslot: Timeslot::parse(&cols[8]),
            building: remove_tag(&cols[9]),
            room: remove_tag(&cols[10]),
        })
    }
}

impl SessionType {
    fn parse(str: &str) -> Option<SessionType> {
        match str {
            "LE" => Some(Self::Lecture),
            "DI" => Some(Self::Discussion),
            "FI" => Some(Self::Final),
            "LA" => Some(Self::Lab),
            "MI" => Some(Self::Midterm),
            "TU" => Some(Self::Tutorial),
            "SE" => Some(Self::Seminar),
            _ => None,
        }
    }
}
impl Day {
    fn parse(str: &str) -> Option<Day> {
        match str {
            "M" => Some(Self::Monday),
            "Tu" => Some(Self::Tuesday),
            "W" => Some(Self::Wednesday),
            "Th" => Some(Self::Thursday),
            "F" => Some(Self::Friday),
            "Sa" => Some(Self::Saturady),
            "Su" => Some(Self::Sunday),
            _ => None,
        }
    }
}
impl Days {
    fn parse(str: &str) -> Option<Days> {
        // If str contains a /, it must be a date
        if str.contains("/") {
            // remove day of week
            let str = str.split_once(" ").unwrap().1;
            return NaiveDate::parse_from_str(str, "%m/%d/%Y")
                .ok()
                .map(|date| Days::Date(date));
        }

        // Otherwise, multiple days
        let mut days = Vec::new();
        let mut cur = String::new();
        for c in str.chars() {
            cur.push(c);
            if let Some(day) = Day::parse(&cur) {
                days.push(day);
                cur.clear();
            }
        }

        Some(Days::Days(days))
    }
}

impl Timeslot {
    fn parse(str: &str) -> Option<Timeslot> {
        fn parse_err(str: &str) -> Result<Timeslot> {
            let str = str.trim();
            let (start, end) = str.split_once("-").ok_or(anyhow!("Invalid timeslot"))?;

            Ok(Timeslot {
                start: NaiveTime::parse_from_str(&(String::from(start) + "m"), "%I:%M%p")?,
                end: NaiveTime::parse_from_str(&(String::from(end) + "m"), "%I:%M%p")?,
            })
        }

        parse_err(str).ok()
    }
}

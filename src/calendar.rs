use chrono::{Datelike, NaiveDate, Weekday};
use ical::{
    generator::{IcalEvent, IcalEventBuilder, Property},
    ical_property,
};

use crate::types::{Course, Day, Days};

impl Into<Vec<IcalEvent>> for &Course {
    fn into(self) -> Vec<IcalEvent> {
        self.sessions
            .iter()
            .enumerate()
            .filter_map(|(i, session)| {
                // Init event
                let event = IcalEventBuilder::tzid("America/Los_Angeles")
                    .uid(format!("{}-{}", self.uuid, i))
                    .changed(
                        chrono::Local::now()
                            .naive_local()
                            .format("%Y%m%dT%H%M%S")
                            .to_string(),
                    );

                // set start and end times
                if session.timeslot.is_none() {
                    return None;
                }

                let timeslot = session.timeslot.clone().unwrap();

                let day = match &session.days {
                    Days::Date(date) => date.clone(),
                    Days::Days(days) => start_date_on_day(
                        self.start_date,
                        days.iter().min().unwrap_or(&Day::Monday),
                    ),
                }
                .format("%Y%m%d");

                let start = timeslot.start.format("%H%M%S");
                let end = timeslot.end.format("%H%M%S");

                let event = event
                    .start(format!("{}T{}", day, start))
                    .end(format!("{}T{}", day, end));

                // Add title based on session type
                let event = event.set(ical_property!(
                    "SUMMARY",
                    format!("{} {:?}", self.code, session.session_type)
                ));

                let event = match &session.days {
                    Days::Days(days) => event.repeat_rule(format!(
                        "FREQ=WEEKLY;WKST=SU;UNTIL={};BYDAY={}",
                        self.end_date.format("%Y%m%d"),
                        days.iter()
                            .map(|day| day.calendar_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    )),
                    _ => event,
                };

                // Make solid on google calendar and add location
                let event = event
                    .set(ical_property!("TRANSP", "OPAQUE"))
                    .set(ical_property!(
                        "LOCATION",
                        format!("{} {}", session.building, session.room)
                    ));

                Some(event.build())
            })
            .collect::<Vec<_>>()
    }
}

fn start_date_on_day(start: NaiveDate, day: &Day) -> NaiveDate {
    let day: Weekday = day.into();
    let diff = day.num_days_from_monday() - start.weekday().num_days_from_monday();
    let new_day = start
        .checked_add_days(chrono::Days::new(diff.into()))
        .unwrap_or(start);
    new_day
}

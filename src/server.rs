use axum::{
    extract::Multipart,
    http::HeaderMap,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use chrono::NaiveDate;
use ical::generator::{Emitter, IcalCalendarBuilder, IcalEvent};

use crate::types::Course;

async fn index() -> impl IntoResponse {
    Html(
        r#"
        <!doctype html>
        <html>
        <head>
            <title>Webreg to ics</title>

            <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/css/bootstrap.min.css" rel="stylesheet" integrity="sha384-T3c6CoIi6uLrA9TneNEoa7RxnatzjcDSCmG1MXxSR1GAsXEV/Dwwykc2MPK8M2HN" crossorigin="anonymous">
        </head>
        <body class="container">
            <br />
            <h1>UCSD Webreg to Calendar</h1>
            <p>Converts an html download from the webreg list view to a iCal file that can be imported to Google Calendar.</p>
            <p>Uses repeating events to avoid flooding calendar with events</p>
            <p>Includes all scheduled course meetings including exams.</p>
            <p>Upload a html file that is from <a href="https://act.ucsd.edu/webreg2/start" target="_blank">webreg</a> after you select a term (stay in list view). Make sure to select full page contents or the equivalent for your browser.</p>
            <hr />
            <form class="d-grid gap-3" method="post" enctype="multipart/form-data">
                <label class="form-label" for="startDate">Term start date (this is winter 24)</label>
                <input class="form-control" type="date" id="startDate" name="startDate" value="2024-01-08" />

                <label class="form-lable" for="endDate">Term end date</label>
                <input class="form-control" type="date" id="endDate" name="endDate" value="2024-03-16" />

                <label class="form-label" for="file">webregMain.html file</label>
                <input class="form-control" type="file" id="file" name="file" />

                <button class="btn btn-primary" type="submit">Upload</button>
            </form>
            <br />
            <p class="italic">Please send any feedback to scottsemtner [at] gmail.com</p>
        </body>
        </html>
        "#,
    )
}

async fn generate_calendar(mut multipart: Multipart) -> impl IntoResponse {
    let mut start_date = NaiveDate::default();
    let mut end_date = NaiveDate::default();
    let mut html = String::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        match name.as_str() {
            "startDate" => {
                start_date =
                    NaiveDate::parse_from_str(&data.escape_ascii().to_string(), "%Y-%m-%d")
                        .unwrap();
            }
            "endDate" => {
                end_date = NaiveDate::parse_from_str(&data.escape_ascii().to_string(), "%Y-%m-%d")
                    .unwrap();
            }
            "file" => {
                html = String::from_utf8_lossy(&data.to_vec()).to_string();
            }
            _ => (),
        }
    }

    let courses = Course::from_html(&html, start_date, end_date).unwrap();
    let mut cal = IcalCalendarBuilder::version("2.0")
        .gregorian()
        .prodid("-//ical-rs//github.com//")
        .build();
    cal.events.extend(
        courses
            .iter()
            .map(|course| Into::<Vec<IcalEvent>>::into(course))
            .flatten()
            .collect::<Vec<_>>(),
    );

    let text = cal.generate();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/octet-stream".parse().unwrap());
    headers.insert(
        "Content-Disposition",
        "attachment; filename=courses.ics".parse().unwrap(),
    );

    (headers, text)
}

pub async fn serve() {
    let app = Router::new()
        .route("/", get(index))
        .route("/", post(generate_calendar));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

use jiff::civil::Date;
use log::error;
use scraper::{ElementRef, Selector};

use crate::error::ScScrapingError;

enum OtherValue {
    Enlisted(String),
    Languages(Vec<String>),
    Location(Vec<String>),
}

pub(crate) struct OtherData {
    pub enlisted: Date,
    pub location: Vec<String>,
    pub languages: Vec<String>,
}

pub(crate) fn parse_other<'a>(element: &ElementRef) -> Result<OtherData, ScScrapingError<'a>> {
    let selector = Selector::parse("div.profile-content > div.left-col > div").unwrap();

    let fragment = element.select(&selector).next().ok_or_else(|| {
        let message = "Other content is empty";
        error!("{message}");
        ScScrapingError::citizen(message)
    })?;

    parse_data(&fragment)
}

fn parse_data<'a>(element: &ElementRef) -> Result<OtherData, ScScrapingError<'a>> {
    let element_selector = Selector::parse("p").unwrap();
    let span_selector = Selector::parse("span").unwrap();
    let strong_selector = Selector::parse("strong").unwrap();

    let iter = element.select(&element_selector).filter_map(|e| {
        let label = e
            .select(&span_selector)
            .map(|l| l.inner_html().trim().to_lowercase())
            .next()?;

        let value = e
            .select(&strong_selector)
            .map(|s| s.inner_html().trim().to_string())
            .next()?;

        match label.as_ref() {
            "enlisted" => Some(OtherValue::Enlisted(value)),
            "location" => {
                let values = value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<_>>();

                if values.is_empty() {
                    None
                } else {
                    Some(OtherValue::Location(values))
                }
            }
            "fluency" => {
                let values = value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<_>>();

                if values.is_empty() {
                    None
                } else {
                    Some(OtherValue::Languages(values))
                }
            }
            _ => None,
        }
    });

    let mut enlisted = None;
    let mut location = Vec::new();
    let mut languages = Vec::new();

    for item in iter {
        match item {
            OtherValue::Enlisted(v) => {
                if enlisted.is_none() {
                    enlisted = Some(v);
                } else {
                    let message = "Found multiple instances of `Enlisted`";
                    error!("{message}");
                    return Err(ScScrapingError::citizen(message));
                }
            }
            OtherValue::Location(v) => {
                if location.is_empty() {
                    location = v;
                } else {
                    let message = "Found multiple instances of `Location`";
                    error!("{message}");
                    return Err(ScScrapingError::citizen(message));
                }
            }
            OtherValue::Languages(v) => {
                if languages.is_empty() {
                    languages = v;
                } else {
                    let message = "Found multiple instances of `Fluency`";
                    error!("{message}");
                    return Err(ScScrapingError::citizen(message));
                }
            }
        }
    }

    let Some(enlisted_value) = enlisted else {
        let message = "Enlisted date not found";
        error!("{message}");
        return Err(ScScrapingError::citizen(message));
    };

    let enlisted_parsed = Date::strptime("%b %d, %Y", &enlisted_value).map_err(|e| {
        let message = format!("Error parsing the date `{enlisted_value}`: {e}");

        error!("{message}");
        ScScrapingError::citizen(message)
    })?;

    let data = OtherData {
        enlisted: enlisted_parsed,
        location,
        languages,
    };

    Ok(data)
}

pub(crate) fn parse_citizen_record_id(element: &ElementRef) -> Option<u64> {
    let selector = Selector::parse("p.citizen-record > strong.value").unwrap();

    element
        .select(&selector)
        .map(|r| r.inner_html())
        .find_map(|id| {
            if let Some(stripped) = id.strip_prefix('#') {
                stripped.parse().ok()
            } else {
                id.parse().ok()
            }
        })
}

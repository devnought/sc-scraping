use log::error;
use scraper::{ElementRef, Selector};

use crate::error::ScScrapingError;

pub(crate) struct WebBio {
    pub website: Option<String>,
    pub bio: Vec<String>,
}

pub(crate) fn parse_web_bio<'a>(element: &ElementRef) -> Result<WebBio, ScScrapingError<'a>> {
    let selector = Selector::parse("div.profile-content > div.right-col").unwrap();

    let fragment = element.select(&selector).next().ok_or_else(|| {
        let message = "Web and bio parent could not be found";
        error!("{message}");
        ScScrapingError::citizen(message)
    })?;

    let web_bio = WebBio {
        website: parse_website(&fragment),
        bio: parse_bio(&fragment),
    };

    Ok(web_bio)
}

fn parse_website(element: &ElementRef) -> Option<String> {
    let selector = Selector::parse("p.website > a").unwrap();

    element.select(&selector).map(|w| w.inner_html()).next()
}

fn parse_bio(element: &ElementRef) -> Vec<String> {
    let selector = Selector::parse("div.entry.bio > div.value").unwrap();

    element
        .select(&selector)
        .flat_map(|e| e.children())
        .filter_map(|e| e.value().as_text())
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
}

use log::error;
use scraper::{ElementRef, Selector};
use url::Url;

use crate::{Title, error::ScScrapingError};

pub(crate) struct ProfileInfo {
    pub avatar: Url,
    pub handle: String,
    pub moniker: String,
    pub title: Title,
}

pub(crate) fn parse_profile_info<'a>(
    element: &ElementRef,
) -> Result<ProfileInfo, ScScrapingError<'a>> {
    let selector = Selector::parse("div.profile.left-col > div").unwrap();
    let fragment = element.select(&selector).next().ok_or_else(|| {
        let message = "Profile content is empty";
        error!("{message}");
        ScScrapingError::citizen(message)
    })?;

    let (moniker, handle, title) = parse_data(&fragment)?;

    let profile = ProfileInfo {
        avatar: parse_avatar(&fragment)?,
        handle,
        moniker,
        title,
    };

    Ok(profile)
}

fn parse_avatar<'a>(element: &ElementRef) -> Result<Url, ScScrapingError<'a>> {
    let selector = Selector::parse("div.thumb > img").unwrap();

    let avatar = element
        .select(&selector)
        .filter_map(|img| img.value().attr("src"))
        .map(|src| format!("https://robertsspaceindustries.com{src}"))
        .find_map(|addr| Url::parse(&addr).ok())
        .ok_or_else(|| {
            let message = "Could not parse avatar URL";
            error!("{message}");
            ScScrapingError::citizen(message)
        })?;

    Ok(avatar)
}

fn parse_data<'a>(element: &ElementRef) -> Result<(String, String, Title), ScScrapingError<'a>> {
    let selector = Selector::parse("div.info > p").unwrap();

    let elements = element.select(&selector).collect::<Vec<_>>();

    if elements.len() != 3 {
        let message = format!("Found {} profile element(s), expected 3.", elements.len());
        error!("{message}");
        return Err(ScScrapingError::citizen(message));
    }

    let strong_selector = Selector::parse("strong").unwrap();

    let moniker = elements[0]
        .select(&strong_selector)
        .map(|s| s.inner_html())
        .next()
        .ok_or_else(|| {
            let message = "Moniker could not be parsed";
            error!("{message}");
            ScScrapingError::citizen(message)
        })?;

    let handle = elements[1]
        .select(&strong_selector)
        .map(|s| s.inner_html())
        .next()
        .ok_or_else(|| {
            let message = "Handle could not be parsed";
            error!("{message}");
            ScScrapingError::citizen(message)
        })?;

    let icon_selector = Selector::parse("span.icon > img").unwrap();
    let title_selector = Selector::parse("span.value").unwrap();

    let title_icon = elements[2]
        .select(&icon_selector)
        .filter_map(|i| i.value().attr("src"))
        .find_map(|src| {
            if src.starts_with("https://") {
                Url::parse(src).ok()
            } else if src.starts_with('/') {
                let new_url = format!("https://robertsspaceindustries.com{src}");
                Url::parse(&new_url).ok()
            } else {
                None
            }
        })
        .ok_or_else(|| {
            let message = "Title icon could not be parsed";
            error!("{message}");
            ScScrapingError::citizen(message)
        })?;

    let title_value = elements[2]
        .select(&title_selector)
        .map(|t| t.inner_html())
        .next()
        .ok_or_else(|| {
            let message = "Title could not be parsed";
            error!("{message}");
            ScScrapingError::citizen(message)
        })?;

    let title = Title {
        icon: title_icon,
        value: title_value,
    };

    Ok((moniker, handle, title))
}

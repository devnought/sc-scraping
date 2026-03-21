use log::{error, warn};
use scraper::{ElementRef, Selector};
use url::Url;

use crate::{error::ScScrapingError, Organization, OrganizationRank, OrganizationType};

struct OrgData {
    name: String,
    sid: String,
    title: String,
    rank: u8,
    url: Url,
    member_count: u64,
}

pub(crate) fn parse_org<'a>(
    element: &ElementRef,
) -> Result<(OrganizationType, Organization), ScScrapingError<'a>> {
    let redacted_selector = Selector::parse("div.member-visibility-restriction").unwrap();

    let org_type = parse_type(element)?;

    let org = if element.select(&redacted_selector).next().is_none() {
        let container_selector = Selector::parse("div.inner-bg > div.left-col > div").unwrap();
        let container = element.select(&container_selector).next().ok_or_else(|| {
            let message = "Could not find organization container";
            error!("{message}");
            ScScrapingError::organization(message)
        })?;

        let logo = parse_logo(&container)?;
        let info = parse_info(&container)?;

        Organization::Visible {
            logo: Box::new(logo),
            name: info.name,
            rank: OrganizationRank {
                name: info.title,
                value: info.rank,
            },
            sid: info.sid,
            url: Box::new(info.url),
            member_count: info.member_count,
        }
    } else {
        Organization::Redacted
    };

    Ok((org_type, org))
}

fn parse_logo<'a>(element: &ElementRef) -> Result<Url, ScScrapingError<'a>> {
    let selector = Selector::parse("div.thumb > a > img").unwrap();

    element
        .select(&selector)
        .filter_map(|i| i.value().attr("src"))
        .map(|src| format!("https://robertsspaceindustries.com{src}"))
        .filter_map(|src| Url::parse(&src).ok())
        .next()
        .ok_or_else(|| {
            let message = "Could not find organization logo";
            error!("{message}");
            ScScrapingError::organization(message)
        })
}

fn parse_info<'a>(element: &ElementRef) -> Result<OrgData, ScScrapingError<'a>> {
    let info_selector = Selector::parse("div.info > p").unwrap();
    let rank_selector = Selector::parse("div.info > div.ranking > span.active").unwrap();
    let member_count_selector = Selector::parse("div.thumb > span.members").unwrap();

    let rank = element.select(&rank_selector).count() as u8;

    let elements = element.select(&info_selector).collect::<Vec<_>>();

    if elements.len() != 3 {
        let message = format!(
            "Found {} organization element(s), expected 3.",
            elements.len()
        );

        error!("{message}");

        return Err(ScScrapingError::organization(message));
    }

    let name_selector = Selector::parse("a").unwrap();
    let (url, name) = elements[0]
        .select(&name_selector)
        .map(|a| {
            let href = a.value().attr("href");

            if href.is_none() {
                let message = "Could not find organization link source";
                error!("{message}");
                return Err(ScScrapingError::organization(message));
            }

            let url_string = format!("https://robertsspaceindustries.com{}", href.unwrap());
            let url = Url::parse(&url_string).ok();

            if url.is_none() {
                let message = "Could not parse organization link `{url_string}`";
                error!("{message}");
                return Err(ScScrapingError::organization(message));
            }

            Ok((url.unwrap(), a.inner_html()))
        })
        .next()
        .ok_or_else(|| {
            let message = "Could not find organization name";
            error!("{message}");
            ScScrapingError::organization(message)
        })??;

    let value_selector = Selector::parse("strong").unwrap();

    let sid = elements[1]
        .select(&value_selector)
        .map(|s| s.inner_html())
        .next()
        .ok_or_else(|| {
            let message = "Could not find organization SID";
            error!("{message}");
            ScScrapingError::organization(message)
        })?;

    let title = elements[2]
        .select(&value_selector)
        .map(|s| s.inner_html())
        .next()
        .ok_or_else(|| {
            let message = "Could not find organization rank";
            error!("{message}");
            ScScrapingError::organization(message)
        })?;

    let member_count = element
        .select(&member_count_selector)
        .next()
        .ok_or_else(|| {
            let message = "Could not find member count";
            error!("{message}");
            ScScrapingError::organization(message)
        })?
        .inner_html()
        .split(' ')
        .next()
        .ok_or_else(|| {
            let message = "Could not split out member count";
            error!("{message}");
            ScScrapingError::organization(message)
        })?
        .parse()
        .map_err(|e| {
            let message = format!("Could not parse organiztion member count: {e}");
            error!("{message}");
            ScScrapingError::organization(message)
        })?;

    let data = OrgData {
        name,
        rank,
        sid,
        title,
        url,
        member_count,
    };

    Ok(data)
}

fn parse_type<'a>(element: &ElementRef) -> Result<OrganizationType, ScScrapingError<'a>> {
    let selector = Selector::parse("div.title").unwrap();

    let org_type = element
        .select(&selector)
        .map(|t| t.inner_html().trim().to_lowercase())
        .next()
        .ok_or_else(|| {
            let message = "Could not find organization type";
            error!("{message}");
            ScScrapingError::organization(message)
        })?;

    match org_type.as_ref() {
        "affiliation" => Ok(OrganizationType::Affiliate),
        "main organization" => Ok(OrganizationType::Main),
        t => {
            let message = format!("Invalid organization type `{t}`");
            error!("{message}");
            Err(ScScrapingError::organization(message))
        }
    }
}

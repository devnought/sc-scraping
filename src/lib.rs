use std::collections::HashMap;

use jiff::civil::Date;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tracing::error;
use url::Url;

mod error;
mod parse_citizen_other;
mod parse_citizen_profile;
mod parse_org;
mod parse_web_bio;

pub use error::*;

use crate::{
    parse_citizen_other::{parse_citizen_probation, parse_citizen_record_id, parse_other},
    parse_citizen_profile::parse_profile_info,
    parse_org::parse_org,
    parse_web_bio::parse_web_bio,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Citizen {
    pub moniker: String,
    pub handle: String,
    pub title: Title,
    pub enlisted: Date,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citizen_record_number: Option<u64>,
    pub avatar: Url,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub location: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fluency: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bio: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probation: Option<Date>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Title {
    pub icon: Url,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Organization {
    Visible {
        logo: Box<Url>,
        name: String,
        sid: String,
        url: Box<Url>,
        rank: OrganizationRank,
        member_count: u64,
    },
    Redacted,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationRank {
    pub name: String,
    pub value: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Organizations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main: Option<Organization>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affiliates: Vec<Organization>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub(crate) enum OrganizationType {
    Main,
    Affiliate,
}

pub fn parse_citizen_record<'a>(html_data: &str) -> Result<Citizen, ScScrapingError<'a>> {
    let html = Html::parse_document(html_data);

    let citizen_selector = Selector::parse("div.public-profile").unwrap();
    let citizen = html.select(&citizen_selector).next().ok_or_else(|| {
        let message = "Citizen content is empty";
        error!("{message}");
        ScScrapingError::citizen(message)
    })?;

    let web_bio = parse_web_bio(&citizen)?;
    let profile_info = parse_profile_info(&citizen)?;
    let other_data = parse_other(&citizen)?;

    let record = Citizen {
        moniker: profile_info.moniker,
        handle: profile_info.handle,
        title: profile_info.title,
        avatar: profile_info.avatar,

        website: web_bio.website,
        bio: web_bio.bio,

        enlisted: other_data.enlisted,
        location: other_data.location,
        fluency: other_data.languages,

        citizen_record_number: parse_citizen_record_id(&citizen),

        probation: parse_citizen_probation(&citizen),
    };

    Ok(record)
}

pub fn parse_organizations<'a>(
    html_data: &str,
) -> Result<Option<Organizations>, ScScrapingError<'a>> {
    let html = Html::parse_document(html_data);

    let org_selector = Selector::parse("div.box-content.org").unwrap();

    let mut map = HashMap::new();

    for e in html.select(&org_selector) {
        let (org_type, org) = parse_org(&e)?;

        if !map.contains_key(&org_type) {
            map.insert(org_type.clone(), Vec::new());
        }

        map.get_mut(&org_type).unwrap().push(org);
    }

    if map.is_empty() {
        return Ok(None);
    }

    if map.contains_key(&OrganizationType::Main)
        && let Some(main) = map.get(&OrganizationType::Main)
        && main.len() > 1
    {
        let message = "Cannot have multiple main organizations";
        error!("{message}");
        return Err(ScScrapingError::organization(message));
    }

    let orgs = Organizations {
        main: map
            .remove(&OrganizationType::Main)
            .map(|mut o| o.pop().unwrap()),

        affiliates: map.remove(&OrganizationType::Affiliate).unwrap_or_default(),
    };

    Ok(Some(orgs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let test_profile_data = include_str!("./test-data/test-profile-probation.html");
        let test_org_data = include_str!("./test-data/test-orgs.html");

        let citizen = parse_citizen_record(test_profile_data).unwrap();
        let orgs = parse_organizations(test_org_data).unwrap();

        // TODO: make this test not suck

        println!("{citizen:#?}");
        println!("{orgs:#?}");
    }
}

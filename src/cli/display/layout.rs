use peertube_api::Video;
use std::fmt;

use super::helpers::*;

pub enum VideoLayoutItem {
    Style(Box<dyn fmt::Display>),
    Name,
    Channel,
    Host,
    Nsfw,
    Views,
    Duration,
    Alignement,
    Published,
    String(String),
}

impl VideoLayoutItem {
    pub fn display(&self, v: &Video) -> String {
        match self {
            VideoLayoutItem::Style(_) => panic!("Internal Error: cannot display style here"),
            VideoLayoutItem::Name => v.name().to_owned(),
            VideoLayoutItem::Channel => v.channel_display().to_owned(),
            VideoLayoutItem::Host => v.host().to_owned(),
            VideoLayoutItem::Nsfw => {
                if v.nsfw() {
                    "nsfw".to_string()
                } else {
                    "".to_string()
                }
            }
            VideoLayoutItem::Views => display_count(v.views()),
            VideoLayoutItem::Duration => pretty_duration(v.duration()),
            VideoLayoutItem::Published => pretty_date(v.published()),
            VideoLayoutItem::String(s) => s.clone(),
            VideoLayoutItem::Alignement => {
                panic!("Internal error, trying to display an alignement")
            }
        }
    }

    pub fn display_as_style(&self) -> String {
        if let VideoLayoutItem::Style(c) = self {
            format!("{}", c)
        } else {
            panic!("Internal error: display as color on other type");
        }
    }

    pub fn is_align(&self) -> bool {
        if let VideoLayoutItem::Alignement = self {
            true
        } else {
            false
        }
    }

    pub fn is_style(&self) -> bool {
        if let VideoLayoutItem::Style(_) = self {
            true
        } else {
            false
        }
    }
}

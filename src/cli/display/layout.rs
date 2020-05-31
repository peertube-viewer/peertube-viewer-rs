use peertube_api::Video;
use std::fmt;

use super::helpers::*;

pub enum LayoutItem<I: InnerLayoutItem> {
    Style(Box<dyn fmt::Display>),
    Alignement,
    Inner(I),
}
impl<D, I> LayoutItem<I>
where
    I: InnerLayoutItem<Data = D>,
{
    pub fn display(&self, data: &I::Data) -> String {
        match self {
            LayoutItem::Style(_) => panic!("Internal Error: cannot display style here"),
            LayoutItem::Alignement => panic!("Internal error, trying to display an alignement"),
            LayoutItem::Inner(i) => i.display(data),
        }
    }

    pub fn display_as_style(&self) -> String {
        if let LayoutItem::Style(c) = self {
            format!("{}", c)
        } else {
            panic!("Internal error: display as color on other type");
        }
    }

    pub fn is_align(&self) -> bool {
        if let LayoutItem::Alignement = self {
            true
        } else {
            false
        }
    }

    pub fn is_style(&self) -> bool {
        if let LayoutItem::Style(_) = self {
            true
        } else {
            false
        }
    }
}

pub enum VideoLayoutItem {
    Name,
    Channel,
    Host,
    Nsfw,
    Views,
    Duration,
    Published,
    String(String),
}

impl InnerLayoutItem for VideoLayoutItem {
    type Data = Video;

    fn display(&self, v: &Self::Data) -> String {
        match self {
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
        }
    }
}

pub trait InnerLayoutItem {
    type Data;
    fn display(&self, data: &Self::Data) -> String;
}

use peertube_api::{channels::Channel, Video};
use std::fmt;
use termion::{color, style};

use super::helpers::*;

pub trait InnerLayoutItem {
    type Data;
    fn display(&self, data: &Self::Data) -> String;
}

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

pub enum ChannelLayoutItem {
    Name,
    Host,
    Followers,
    String(String),
}

impl InnerLayoutItem for ChannelLayoutItem {
    type Data = Channel;

    fn display(&self, c: &Self::Data) -> String {
        match self {
            ChannelLayoutItem::Name => c.display_name().to_owned(),
            ChannelLayoutItem::Host => c.host().to_owned(),
            ChannelLayoutItem::Followers => display_count(c.followers()),
            ChannelLayoutItem::String(s) => s.clone(),
        }
    }
}

pub fn default_video_layouts() -> (
    Vec<LayoutItem<VideoLayoutItem>>,
    Vec<LayoutItem<VideoLayoutItem>>,
) {
    let video_layout = vec![
        LayoutItem::Style(Box::new(color::Fg(color::Blue))),
        LayoutItem::Inner(VideoLayoutItem::Name),
        LayoutItem::Inner(VideoLayoutItem::String(" ".to_string())),
        LayoutItem::Alignement,
        LayoutItem::Style(Box::new(color::Fg(color::Green))),
        LayoutItem::Inner(VideoLayoutItem::Channel),
        LayoutItem::Inner(VideoLayoutItem::String(" ".to_string())),
        LayoutItem::Alignement,
        LayoutItem::Style(Box::new(color::Fg(color::Cyan))),
        LayoutItem::Inner(VideoLayoutItem::Host),
        LayoutItem::Alignement,
        LayoutItem::Style(Box::new(color::Fg(color::Yellow))),
        LayoutItem::Inner(VideoLayoutItem::String(" [".to_string())),
        LayoutItem::Inner(VideoLayoutItem::Duration),
        LayoutItem::Inner(VideoLayoutItem::String("] ".to_string())),
        LayoutItem::Alignement,
        LayoutItem::Style(Box::new(color::Fg(color::Green))),
        LayoutItem::Inner(VideoLayoutItem::Views),
        LayoutItem::Inner(VideoLayoutItem::String(" ".to_string())),
        LayoutItem::Alignement,
        LayoutItem::Inner(VideoLayoutItem::Published),
        LayoutItem::Inner(VideoLayoutItem::String(" ".to_string())),
        LayoutItem::Alignement,
        LayoutItem::Style(Box::new(color::Fg(color::Red))),
        LayoutItem::Inner(VideoLayoutItem::Nsfw),
        LayoutItem::Style(Box::new(color::Fg(color::Reset))),
    ];

    let seen_video_layout = vec![
        LayoutItem::Style(Box::new(style::Bold)),
        LayoutItem::Inner(VideoLayoutItem::Name),
        LayoutItem::Style(Box::new(style::Reset)),
        LayoutItem::Inner(VideoLayoutItem::String(" ".to_string())),
        LayoutItem::Alignement,
        LayoutItem::Style(Box::new(color::Fg(color::Green))),
        LayoutItem::Inner(VideoLayoutItem::Channel),
        LayoutItem::Inner(VideoLayoutItem::String(" ".to_string())),
        LayoutItem::Alignement,
        LayoutItem::Style(Box::new(color::Fg(color::Cyan))),
        LayoutItem::Inner(VideoLayoutItem::Host),
        LayoutItem::Alignement,
        LayoutItem::Style(Box::new(color::Fg(color::Yellow))),
        LayoutItem::Inner(VideoLayoutItem::String(" [".to_string())),
        LayoutItem::Inner(VideoLayoutItem::Duration),
        LayoutItem::Inner(VideoLayoutItem::String("] ".to_string())),
        LayoutItem::Alignement,
        LayoutItem::Style(Box::new(color::Fg(color::Green))),
        LayoutItem::Inner(VideoLayoutItem::Views),
        LayoutItem::Inner(VideoLayoutItem::String(" ".to_string())),
        LayoutItem::Alignement,
        LayoutItem::Inner(VideoLayoutItem::Published),
        LayoutItem::Inner(VideoLayoutItem::String(" ".to_string())),
        LayoutItem::Alignement,
        LayoutItem::Style(Box::new(color::Fg(color::Red))),
        LayoutItem::Inner(VideoLayoutItem::Nsfw),
        LayoutItem::Style(Box::new(color::Fg(color::Reset))),
    ];

    (video_layout, seen_video_layout)
}

pub fn default_channel_layouts() -> Vec<LayoutItem<ChannelLayoutItem>> {
    vec![
        LayoutItem::Style(Box::new(color::Fg(color::Blue))),
        LayoutItem::Inner(ChannelLayoutItem::Name),
        LayoutItem::Inner(ChannelLayoutItem::String(" ".to_string())),
        LayoutItem::Alignement,
        LayoutItem::Style(Box::new(color::Fg(color::Cyan))),
        LayoutItem::Inner(ChannelLayoutItem::Host),
        LayoutItem::Alignement,
        LayoutItem::Inner(ChannelLayoutItem::String(" ".to_string())),
        LayoutItem::Style(Box::new(color::Fg(color::Green))),
        LayoutItem::Inner(ChannelLayoutItem::Followers),
        LayoutItem::Style(Box::new(color::Fg(color::Reset))),
    ]
}

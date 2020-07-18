use peertube_api::{channels::Channel, Comment, Resolution, Video};

use super::{
    config::Blocklist,
    history::{History, HistoryT},
};
use std::fmt::{self, Debug};
use termion::{color, style};

use std::cmp;
use std::rc::Rc;

mod layout;
use layout::{
    default_channel_layouts, default_comment_layouts, default_video_layouts, ChannelLayoutItem,
    CommentLayoutItem, InnerLayoutItem, LayoutItem, VideoLayoutItem,
};

mod helpers;
use helpers::*;

use unicode_width::UnicodeWidthStr;

const DEFAULT_COLS: usize = 20;

pub struct Display {
    cols: usize,
    colors: bool,
    video_layout: Vec<LayoutItem<VideoLayoutItem>>,
    seen_video_layout: Vec<LayoutItem<VideoLayoutItem>>,
    channel_layout: Vec<LayoutItem<ChannelLayoutItem>>,
    comment_layout: Vec<LayoutItem<CommentLayoutItem>>,
}

#[derive(Debug)]
enum MaybeColor<T: color::Color> {
    No,
    Fg(color::Fg<T>),
}

impl<T: color::Color> fmt::Display for MaybeColor<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaybeColor::No => write!(f, ""),
            MaybeColor::Fg(c) => write!(f, "{}", c),
        }
    }
}

impl Display {
    pub fn new(colors: bool) -> Display {
        let cols = termion::terminal_size()
            .map(|(c, _r)| c as usize)
            .unwrap_or(DEFAULT_COLS);

        let (video_layout, seen_video_layout) = default_video_layouts();

        Display {
            cols,
            colors,
            video_layout,
            seen_video_layout,
            channel_layout: default_channel_layouts(),
            comment_layout: default_comment_layouts(),
        }
    }

    /// Abstract removing colors
    /// This should always be used instead of termions colors for displaying output
    fn fg_color<C: color::Color>(&self, c: C) -> MaybeColor<C> {
        if self.colors {
            MaybeColor::Fg(color::Fg(c))
        } else {
            MaybeColor::No
        }
    }

    /// Display a list of video results
    pub fn video_list(
        &self,
        videos: &[Rc<Video>],
        history: &History,
        blocklist: &impl Blocklist<Video>,
    ) {
        self.list(
            videos,
            history,
            blocklist,
            &self.video_layout,
            &self.seen_video_layout,
        );
    }

    pub fn channel_list(&self, channels: &[Rc<Channel>], _: &History, _: &impl Blocklist<Video>) {
        self.list(
            channels,
            &(),
            &(),
            &self.channel_layout,
            &self.channel_layout,
        );
    }

    pub fn comment_list(&self, comments: &[Rc<Comment>]) {
        self.list(
            comments,
            &(),
            &(),
            &self.comment_layout,
            &self.comment_layout,
        );
    }

    fn list<I, D, B, H>(
        &self,
        contents: &[Rc<D>],
        history: &H,
        blocklist: &B,
        layout: &[LayoutItem<I>],
        seen_layout: &[LayoutItem<I>],
    ) where
        B: Blocklist<D>,
        H: HistoryT<D>,
        I: InnerLayoutItem<Data = D>,
    {
        let mut content_parts = Vec::new();
        let mut alignements_total = vec![
            0;
            cmp::max(
                seen_layout.iter().filter(|i| i.is_align()).count(),
                layout.iter().filter(|i| i.is_align()).count()
            )
        ];
        for v in contents {
            let mut tmp_str = Vec::new();
            let mut tmp_align = Vec::new();

            if blocklist.is_blocked(&v).is_some() {
                content_parts.push((tmp_str, tmp_align));
                continue;
            }

            let mut align_off: usize = 0;
            let mut align_id = 0;
            let layout_iter = if history.is_viewed(&**v) {
                seen_layout.iter()
            } else {
                layout.iter()
            };

            for item in layout_iter {
                if !item.is_align() && !item.is_style() {
                    let dsp = item.display(v);
                    let s: &str = &dsp;
                    align_off += UnicodeWidthStr::width(s);
                    tmp_str.push(dsp);
                } else if item.is_align() {
                    if alignements_total[align_id] < align_off {
                        alignements_total[align_id] = align_off;
                    }
                    tmp_align.push(align_off);
                    align_off = 0;
                    align_id += 1;
                }
            }
            content_parts.push((tmp_str, tmp_align));
        }

        let mut buffer = String::new();
        for (id, parts) in content_parts.into_iter().enumerate() {
            buffer.push_str(&(id + 1).to_string());
            buffer.push_str(
                &" ".to_string()
                    .repeat(display_length(contents.len()) - display_length(id + 1)),
            );
            buffer.push_str(": ");

            if let Some(reason) = blocklist.is_blocked(&contents[id]) {
                buffer.push_str(&format!(
                    "{}{}{}\n",
                    self.fg_color(color::Red),
                    reason,
                    self.fg_color(color::Reset)
                ));
                continue;
            }

            let mut layout_align_it = alignements_total.iter();
            let layout_it = if history.is_viewed(&*contents[id]) {
                seen_layout.iter()
            } else {
                layout.iter()
            };
            let mut parts_it = parts.0.iter();
            let mut parts_align_it = parts.1.iter();

            for item in layout_it {
                if item.is_align() {
                    let spacing = layout_align_it
                        .next()
                        .expect("Internal error: align smaller than expected")
                        - parts_align_it
                            .next()
                            .expect("Internal error: align smaller than expected");
                    buffer.push_str(&" ".to_string().repeat(spacing));
                } else if item.is_style() {
                    if self.colors {
                        buffer.push_str(&item.display_as_style());
                    }
                } else {
                    buffer.push_str(
                        &parts_it
                            .next()
                            .expect("Internal Error: parts smaller than alignement"),
                    );
                }
            }

            buffer.push('\n');
        }

        print!("{}", buffer);
    }

    pub fn resolutions(&self, resolutions: Vec<Resolution>) {
        self.line('=');
        self.print_centered("Resolution selection");
        self.line('=');
        let mut lengths = Vec::new();
        let mut max_len = 10; //Length of "Resolution"
        for r in resolutions.iter() {
            let len = r.label().chars().count();
            if len > max_len {
                max_len = len;
            }
            lengths.push(len);
        }

        println!(
            "{}Resolution{}Size",
            " ".to_string()
                .repeat(display_length(resolutions.len()) + 2),
            " ".to_string().repeat(max_len - 10 + 2),
        );

        for (id, r) in resolutions.iter().enumerate() {
            let spacing = " ".to_string().repeat(max_len - lengths[id]);
            let colon_spacing = " "
                .to_string()
                .repeat(display_length(resolutions.len()) - display_length(id + 1));
            println!(
                "{}{}: {} {} {}",
                id + 1,
                colon_spacing,
                r.label(),
                spacing,
                pretty_size(*r.size()),
            )
        }
    }

    pub fn welcome(&self, instance: &str) {
        self.line('=');
        self.print_centered(&format!("Connecting to: {}", instance));
        self.line('=');
    }

    pub fn err<T: fmt::Display>(&self, err: &T) {
        println!(
            "{}{}{}{}{}",
            self.fg_color(color::Red),
            style::Bold,
            err,
            style::Reset,
            self.fg_color(color::Reset)
        );
    }

    pub fn warn<T: fmt::Display>(&self, warn: &T) {
        println!(
            "{}{}{}{}{}",
            self.fg_color(color::Yellow),
            style::Bold,
            warn,
            style::Reset,
            self.fg_color(color::Reset)
        );
    }

    pub fn message(&self, msg: &str) {
        println!("{}", msg);
    }

    pub fn mode_info(&self, mode: &str, total: usize, offset: usize, current_len: usize) {
        println!(
            "{}{}{} results {} to {} out of {}{}",
            style::Bold,
            style::Underline,
            mode,
            offset,
            offset + current_len,
            total,
            style::Reset
        );
    }

    pub fn continue_despite_error(&self) {
        println!(
            "{}{}You can continue browsing the PeerTube network{}",
            style::Bold,
            style::Underline,
            style::Reset
        );
    }

    pub async fn video_info(&self, video: &Video) {
        self.line('=');
        self.print_centered(video.name());
        self.line('=');
        if let Ok(Some(d)) = video.description().await {
            if !d.is_empty() {
                self.print_centered("DESCRIPTION");
                self.line('=');
                println!("{}", d);
                self.line('=');
            }
        }
        self.print_centered("INFORMATIONS");
        self.line('=');
        println!("duration : {}", pretty_duration(video.duration()));
        println!("views    : {}", video.views());
        println!("likes    : {}", video.likes());
        println!("dislikes : {}", video.dislikes());
        println!("released : {}", full_date(video.published()));
        println!("account  : {}", video.account_display());
        println!("channel  : {}", video.channel_display());
        println!("host     : {}", video.host());
        println!("url      : {}", video.watch_url());
        if video.nsfw() {
            println!("{}nsfw{}", self.fg_color(color::Red), style::Reset,);
        }
        self.line('=');
    }

    pub async fn channel_info(&self, channel: &Channel) {
        self.line('=');
        self.print_centered(channel.display_name());
        self.line('=');
        if let Some(d) = channel.description() {
            if !d.is_empty() {
                self.print_centered("DESCRIPTION");
                self.line('=');
                println!("{}", d);
                self.line('=');
            }
        }
        println!("name          : {}", channel.name());
        println!("display_name  : {}", channel.display_name());
        println!("host          : {}", channel.host());
        println!("followers     : {}", channel.followers());
        println!("created       : {}", channel.created_at());
        println!("handle        : {}", channel.handle());
        println!("rss feed      : {}", channel.rss());
        println!("atom feed     : {}", channel.atom());
        self.line('=');
    }

    pub fn report_error(&self, err: impl Debug) {
        self.message(&format!(
            "\
            However, if you believe that this is a bug from peertube-viewer-rs, please file a bug report at:\n\
            https://gitlab.com/peertube-viewer/peertube-viewer-rs/-/issues\n\
            \n\
            With the following information (you might want to anonymise it before sending it):\n\
            {:?}
",
            err
        ));
    }

    fn line(&self, c: char) {
        let line_str = c.to_string().repeat(self.cols);
        println!("{}", line_str);
    }

    fn print_centered(&self, s: &str) {
        let len = s.chars().count();
        if len > self.cols {
            println!("{}", s);
            return;
        }

        let before = ' '.to_string().repeat((self.cols - len) / 2);
        println!("{}{}", before, s);
    }
}

mod viuer;

use std::fmt::Display;
use std::io;
use clap::{arg, command};
use crossterm::{cursor, execute};
use csscolorparser::Color;
use image::{imageops, DynamicImage, ImageBuffer, Rgba};
use whoami::fallible;
use owo_colors::OwoColorize;

macro_rules! clearScreen {
    ($T:expr) => {
        //clearscreen::clear().unwrap_or(());
        for _ in 0..$T {
            println!();
        }
    };
}

#[derive(Clone)]
struct TermLine {
    label: Option<String>,
    text: String
}
impl Display for TermLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.label.is_some() {
            return write!(f, "{}: {}", self.label.as_ref().unwrap().bright_magenta(), self.text);
        } else {
            return write!(f, "{}", self.text);
        }
    }
}

macro_rules! addLine {
    ($lines:expr, $label:expr, $line:expr) => {
        $lines.push(TermLine {
            label: $label,
            text: $line
        })
    };
}

fn main() {
    #[allow(unused_variables)]
    let (term_size_x, term_size_y) = viuer::terminal_size();
    let name_string = format!(
        "{}@{}",
        whoami::realname(),
        fallible::hostname().unwrap_or_else(|_e| -> String { String::from("?") })
    );
    let mut im_w = ((term_size_x as f32 / 2.0).floor() - (name_string.len() as f32 / 2.0).floor()) as u32;
    if im_w/2 > (term_size_y-2) as u32 {
        im_w = ((term_size_y-2) * 2) as u32
    }
    let mut im_h = 0 as u32;

    let args = command!()
        .version(env!("CARGO_PKG_VERSION"))
        .arg(arg!(-i --im <FILE> "Image to display, defaults to none"))
        .arg(arg!(-b --bgc <COLOR> "Any valid css color"))
        .get_matches();

    let im_path = args.get_one::<String>("im");
    let bg_color = args.get_one::<String>("bgc");
    let has_im = im_path.is_some();
    let mut lines: Vec<TermLine> = vec![];

    clearScreen!(term_size_y);

    //lines.push(name_string.clone());
    //lines.push(format!("╶{:─<1$}╴", "", name_string.len() - 2));
    //lines.push(format!("{} {}", whoami::distro(), whoami::arch()));
    addLine!(lines, None, format!("{}@{}", whoami::realname().bright_magenta(), fallible::hostname().unwrap_or_else(|_e| -> String { String::from("?") }).bright_magenta()));
    addLine!(lines, None, format!("╶{:─<1$}╴", "", name_string.len() - 2));
    addLine!(lines, Some("OS".to_string()), format!("{} {}", whoami::distro(), whoami::arch()));

    if has_im {
        let im = image::open(im_path.unwrap())
            .expect("Image load failed")
            .to_rgba8();

        let conf = viuer::Config {
            // set offset
            x: 0,
            y: 0,
            // set dimensions
            width: Some(im_w),
            transparent: true,
            use_sixel: true,
            ..Default::default()
        };
        let mut bgim = ImageBuffer::from_pixel(
            u32::max(im.width(), im.height()),
            u32::max(im.width(), im.height()),
            Rgba::<u8>([0, 0, 0, 0]),
        );
        if bg_color.is_some() {
            let clr = bg_color.unwrap().parse::<Color>().expect("BG Color error");
            bgim = ImageBuffer::from_pixel(
                u32::max(im.width(), im.height()),
                u32::max(im.width(), im.height()),
                Rgba::<u8>([
                    (clr.r * 255.0) as u8,
                    (clr.g * 255.0) as u8,
                    (clr.b * 255.0) as u8,
                    (clr.a * 255.0) as u8,
                ]),
            );
        }
        if im.width() < im.height() {
            imageops::overlay(&mut bgim, &im, (im.height()/2 - im.width()/2) as i64, 0);
        }
        else if im.width() < im.height() {
            imageops::overlay(&mut bgim, &im, 0, (im.width()/2 - im.height()/2) as i64);
        }
        else {
            imageops::overlay(&mut bgim, &im, 0, 0);
        }

        (im_w, im_h) = viuer::print(&DynamicImage::ImageRgba8(bgim), &conf).expect("Shit");
    } else {
        im_w = 0;
    }

    for (i, line) in lines.clone().iter().enumerate() {
        execute!(
            io::stdout(),
            cursor::MoveTo((im_w + (1 * ((im_w != 0) as u32))) as u16, i as u16)
        )
        .expect("Failed to print line");
        println!("{}", line);
    }

    execute!(
        io::stdout(),
        cursor::MoveTo(
            (im_w - (1 * ((im_w != 0) as u32))) as u16,
            (im_h - (1 * ((im_w != 0) as u32))) as u16
        )
    )
    .expect("shit x2");
    println!("\n")
}

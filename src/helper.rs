use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;

pub fn print_center(msg: String) {
    if let Some(size) = termsize::get() {
        let width = size.cols as usize;

        let msg_width = UnicodeWidthStr::width(&msg[..]); // correct display width
        if msg_width < width {
            let padding = (width.saturating_sub(msg_width)) / 2;
            // print spaces first, then the message
            println!("{:padding$}{}", "", msg, padding = padding);
        } else {
            println!("{msg}"); // fallback if too long
        }
    } else {
        println!("{msg}"); // fallback if can't detect size
    }
    io::stdout().flush().unwrap();
}

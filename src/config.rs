use std::io::{self, Write};

use env_logger::fmt::Formatter;
use log::Record;

pub fn envlogger_write_format(buf: &mut Formatter, rec: &Record) -> io::Result<()> {
    let level_string = format!("{}", rec.level());
    let level_style = buf.default_level_style(rec.level());
    write!(buf, "[")?;
    write!(buf, "{}", level_style.render_reset())?;
    write!(buf, "{}", level_style.render())?;
    write!(buf, "{}", level_string)?;
    write!(buf, "{}", level_style.render_reset())?;
    writeln!(
        buf,
        " @ {}] {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        rec.args()
    )
}

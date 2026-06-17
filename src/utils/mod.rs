pub const LOGO_LINES: [&str; 6] = [
    "__      _____ _   _ _ __  ",
    "\\ \\ /\\ / / __| | | | '_ \\ ",
    " \\ V  V /\\__ \\ |_| | |_) |",
    "  \\_/\\_/ |___/\\__,_| .__/ ",
    "                   | |    ",
    "                   |_|    ",
];

pub fn formatMemory(bytes: u64) -> String {
    let kb = bytes / 1024;
    let mb = kb / 1024;
    if mb > 0 {
        format!("{}M", mb)
    } else {
        format!("{}K", kb)
    }
}
use maud::{html, Markup};

use crate::controllers::directory::DirEntryInfo;
use chrono::{DateTime, Local};

fn format_entry(base_path: &str, entry: &DirEntryInfo) -> Markup {
    let size = if entry.is_dir { "-".to_string() } else { format_size(entry.size) };
    let mtime = format_mtime(entry.mtime);

    html! {
        tr {
            td {
                @if entry.is_dir {
                    svg class="icon" width="20" height="20" {
                        use href="#icon-folder" {}
                    }
                } @else {
                    svg class="icon" width="20" height="20" {
                        use href="#icon-file" {}
                    }
                }
            }
            td {
                @if entry.is_dir {
                    a class="dir" href={(format!("{}{}", base_path, entry.name))} { (entry.name) }
                } @else {
                    a href={(format!("{}{}", base_path, entry.name))} { (entry.name) }
                }
            }
            td { (size) }
            td { (mtime) }
        }
    }
}

pub fn directory_listing(_base_path: &str, entries: &[DirEntryInfo], req_base_path: &str) -> Markup {
    html! {
        html {
            head {
                title { "Directory Listing - Rook LifeWatch Admin" }
                style { r#"
                    table { border-collapse: collapse; width: 99%; margin: 2em auto; }
                    th, td { border: 1px solid #ccc; padding: 0.5em 1em; text-align: left; }
                    th { background: #f0f0f0; }
                    tr:hover { background: #f9f9f9; }
                    .dir { font-weight: bold; }
                "# }
                svg xmlns="http://www.w3.org/2000/svg" style="display:none;" {
                    symbol id="icon-folder" viewBox="0 0 20 20" {
                        rect x="2" y="6" width="16" height="12" rx="2" fill="#FFD700" {}
                        path d="M2 6V4a2 2 0 0 1 2-2h4l2 2h6a2 2 0 0 1 2 2v2" stroke="#B8860B" stroke-width="1.5" fill="none" {}
                    }
                    symbol id="icon-file" viewBox="0 0 20 20" {
                        rect x="4" y="2" width="12" height="16" rx="2" fill="#87CEEB" {}
                        rect x="4" y="2" width="12" height="16" rx="2" stroke="#4682B4" stroke-width="1.5" fill="none" {}
                    }
                    symbol id="icon-uplink" viewBox="0 0 16 16" {
                        path d="M8 12V4M8 4L4 8M8 4L12 8" stroke="#555" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round" {}
                    }
                }
            }
            body {
                h1 { "Directory Listing" }
                table {
                    thead {
                        tr {
                            th { "" }
                            th { "Name" }
                            th { "Size" }
                            th { "Modified" }
                        }
                    }
                    tbody {
                        // Uplink row
                        tr {
                            td {
                                svg class="icon" width="20" height="20" {
                                    use href="#icon-folder" {}
                                }
                            }
                            td {
                                a class="dir" href={(format!("{}..", req_base_path))} {
                                    svg class="icon" width="16" height="16" style="margin-right:4px;" {
                                        use href="#icon-uplink" {}
                                    }
                                    ".."
                                }
                            }
                            td { "" }
                            td { "" }
                        }
                        @for entry in entries {
                            (format_entry(req_base_path, entry))
                        }
                    }
                }
            }
        }
    }
}

fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else {
        format!("{:.1} MB", size as f64 / 1024.0 / 1024.0)
    }
}

fn format_mtime(mtime: u64) -> String {
    DateTime::from_timestamp(mtime as i64, 0)
        .map(|dt| dt.with_timezone(&Local))
        .map_or(String::from("-"), |dt| dt.format("%Y-%m-%d %H:%M").to_string())
}
use crate::RepeaterAtlasError;
use crate::dao::user_location::UserLocation;
use askama::Template;
use std::process::Stdio;
use tempfile::TempDir;
use tokio::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSize {
    A4,
    A5,
}

impl PageSize {
    pub fn as_typst(&self) -> &'static str {
        match self {
            PageSize::A4 => "a4",
            PageSize::A5 => "a5",
        }
    }
}

pub struct LogbookOptions {
    pub call_sign: String,
    pub page_size: PageSize,
    pub log_pages: u32,
    pub title_page: bool,
    pub locations_page: bool,
    pub phonetic_alphabet_page: bool,
    pub frequency_bands_page: bool,
    pub locations: Vec<UserLocation>,
}

pub struct LogbookLocation {
    pub address: String,
    pub maidenhead: String,
    pub latlon: String,
}

#[derive(Template)]
#[template(path = "logbook.typ")]
struct LogbookTemplate<'a> {
    call_sign: &'a str,
    paper: &'a str,
    log_pages: u32,
    title_page: bool,
    phonetic_page: bool,
    bands_page: bool,
    locations_page: bool,
    locations: Vec<LogbookLocation>,
}

pub fn render_typst(opts: &LogbookOptions) -> Result<String, RepeaterAtlasError> {
    let locations = opts
        .locations
        .iter()
        .map(|loc| LogbookLocation {
            address: loc.address.as_deref().unwrap_or("").to_string(),
            maidenhead: loc.maidenhead.as_deref().unwrap_or("").to_string(),
            latlon: match (loc.latitude, loc.longitude) {
                (Some(lat), Some(lon)) => format!("{lat:.5}, {lon:.5}"),
                _ => String::new(),
            },
        })
        .collect();

    LogbookTemplate {
        call_sign: &opts.call_sign,
        paper: opts.page_size.as_typst(),
        log_pages: opts.log_pages,
        title_page: opts.title_page,
        phonetic_page: opts.phonetic_alphabet_page,
        bands_page: opts.frequency_bands_page,
        locations_page: opts.locations_page,
        locations,
    }
    .render()
    .map_err(|e| RepeaterAtlasError::OtherMsg(format!("template render failed: {e}")))
}

pub async fn generate_pdf(opts: &LogbookOptions) -> Result<Vec<u8>, RepeaterAtlasError> {
    let dir = TempDir::new().map_err(RepeaterAtlasError::Io)?;
    let typ_path = dir.path().join("logbook.typ");
    let pdf_path = dir.path().join("logbook.pdf");

    let source = render_typst(opts)?;
    std::fs::write(&typ_path, source.as_bytes()).map_err(RepeaterAtlasError::Io)?;

    let output = Command::new("typst")
        .arg("compile")
        .arg(&typ_path)
        .arg(&pdf_path)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|e| RepeaterAtlasError::OtherMsg(format!("failed to run typst: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RepeaterAtlasError::OtherMsg(format!(
            "typst compile failed: {stderr}"
        )));
    }

    let pdf = std::fs::read(&pdf_path).map_err(RepeaterAtlasError::Io)?;
    Ok(pdf)
}

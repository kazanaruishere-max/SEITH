use anyhow::Result;

pub struct CalendarScraper;

impl CalendarScraper {
    pub fn new() -> Self {
        Self
    }

    pub async fn fetch_events(&self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}

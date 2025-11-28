// TODO use organization, affiliation and location

#[derive(Debug, Clone, Default)]
pub struct Document {
    pub title: Option<String>,
    pub authors: Option<String>,
    pub date: Option<String>,
    pub abstractt: Option<String>,
    pub bibliography: Option<String>,
    pub organization: Option<String>,
    pub affiliation: Option<String>,
    pub location: Option<String>,
    pub content: String,
}

impl Document {
    pub fn to_latex(&self, template: String) -> String {
        let title = self.title.as_deref().unwrap_or("");
        let authors = self.authors.as_deref().unwrap_or("");
        let abstract_text = self.abstractt.as_deref().unwrap_or("");
        let bibliography = self.bibliography.as_deref().unwrap_or("");
        let date = self.date.as_deref().unwrap_or(r"\today");
        let content = &self.content;

        template
            .replace("%title%", title)
            .replace("%authors%", authors)
            .replace("%abstract%", abstract_text)
            .replace("%bibliography%", bibliography)
            .replace("%date%", date)
            .replace("%content%", content)
    }
}

use crate::api;

#[derive(Debug)]
pub struct Hosted {
    pub url: reqwest::Url,
    pub pages: Vec<String>
}

impl Hosted {
    fn new(url: reqwest::Url, pages: Vec<String>) -> Hosted {
        Hosted {
            url,
            pages
        }
    }
}

#[derive(Debug)]
pub struct External {
    pub url: reqwest::Url
}

impl External {
    pub fn new(url: reqwest::Url) -> External {
        External { url }
    }
}

#[derive(Debug)]
pub enum ChapterPages {
    Hosted(Hosted),
    External(External)
}

impl ChapterPages {
    pub fn hosted(url: reqwest::Url, pages: Vec<String>) -> ChapterPages {
        ChapterPages::Hosted(Hosted::new(url, pages))
    }

    pub fn external(url: reqwest::Url) -> ChapterPages {
        ChapterPages::External(External::new(url))
    }
}

#[derive(Debug)]
pub struct Chapter {
    pub id: u64,
    pub volume: String,
    pub chapter: String,
    pub title: String,
    pub manga_id: u64,
    pub pages: ChapterPages
}

impl Chapter {
    pub fn new(id: u64, chapter_api: api::Chapter) -> Chapter {
        Chapter {
            id,
            volume: chapter_api.volume,
            chapter: chapter_api.chapter,
            title: chapter_api.title,
            manga_id: chapter_api.manga_id,
            pages: match chapter_api.external {
                Some(external) => ChapterPages::external(reqwest::Url::parse(&external).unwrap()),
                None => ChapterPages::hosted(
                    reqwest::Url::parse(&chapter_api.server)
                        .or(reqwest::Url::parse("https://mangadex.org/").unwrap().join(&chapter_api.server))
                        .unwrap()
                        .join(&format!("{}/", chapter_api.hash))
                        .unwrap(),
                    chapter_api.page_array
                )
            }
        }
    }
}

impl std::string::ToString for Chapter {
    fn to_string(&self) -> String {
        if self.title == "Oneshot" {
            format!("Oneshot [{}]", self.id)
        }
        else {
            match (self.title.is_empty(), self.volume.is_empty()) {
                (true, true) => sanitize_filename::sanitize(format!("Ch. {} [{}]", self.chapter, self.id)),
                (true, false) => sanitize_filename::sanitize(format!("Vol. {} Ch. {} [{}]", self.volume, self.chapter, self.id)),
                (false, true) => sanitize_filename::sanitize(format!("Ch. {} - {} [{}]", self.chapter, self.title, self.id)),
                _ => sanitize_filename::sanitize(format!(
                    "Vol. {} Ch. {} - {} [{}]",
                    self.volume, self.chapter, self.title, self.id
                )),
            }
        }
    }
}
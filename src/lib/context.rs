use crate::api;
use crate::fs;

pub struct Context {
    pub manga: tokio::sync::RwLock<std::collections::HashMap<u64, std::sync::Arc<fs::entry::Manga>>>,
    pub chapters: tokio::sync::RwLock<std::collections::HashMap<u64, std::sync::Arc<fs::entry::Chapter>>>,
    pub pages: tokio::sync::RwLock<std::collections::HashMap<reqwest::Url, std::sync::Arc<fs::entry::Page>>>,
    pub covers: tokio::sync::RwLock<std::collections::HashMap<reqwest::Url, std::sync::Arc<fs::entry::Cover>>>,

    pub entries: tokio::sync::RwLock<std::collections::HashMap<u64, fs::entry::Inode>>,
    manga_inodes: tokio::sync::RwLock<std::collections::HashMap<u64, u64>>,
    chapters_inodes: tokio::sync::RwLock<std::collections::HashMap<u64, u64>>,
    pages_inodes: tokio::sync::RwLock<std::collections::HashMap<reqwest::Url, u64>>,
    cover_inodes: tokio::sync::RwLock<std::collections::HashMap<reqwest::Url, u64>>,

    server: tokio::sync::Mutex<polyfuse_tokio::Server>,
    api: tokio::sync::RwLock<api::MangaDexAPI>,
    next_ino: tokio::sync::Mutex<u64>,
    uid: nix::unistd::Uid,
    gid: nix::unistd::Gid
}

pub enum GetOrFetch<T> {
    Cached(T),
    Fetched(T)
}

impl<T> GetOrFetch<T> {
    pub fn get(self) -> T {
        match self {
            GetOrFetch::Cached(value) => value,
            GetOrFetch::Fetched(value) => value
        }
    }
}

pub type GetOrFetchRef<T> = GetOrFetch<std::sync::Weak<T>>;

impl Context {
    pub fn new(server: polyfuse_tokio::Server, uid: nix::unistd::Uid, gid: nix::unistd::Gid) -> std::sync::Arc<Context> {
        let mut entries = std::collections::HashMap::new();

        entries.insert(1u64, fs::entry::Inode(fs::entry::Entry::Root(fs::entry::Directory::root()), fs::entry::Attributes::new(1u64, uid.clone(), gid.clone())));

        std::sync::Arc::new(Context {
            server: tokio::sync::Mutex::new(server),
            api: tokio::sync::RwLock::new(api::MangaDexAPI::new()),
            manga: tokio::sync::RwLock::new(std::collections::HashMap::default()),
            chapters: tokio::sync::RwLock::new(std::collections::HashMap::default()),
            pages: tokio::sync::RwLock::new(std::collections::HashMap::default()),
            covers: tokio::sync::RwLock::new(std::collections::HashMap::default()),
            entries: tokio::sync::RwLock::new(entries),
            next_ino: tokio::sync::Mutex::new(2u64),
            uid, gid,
            manga_inodes: tokio::sync::RwLock::default(),
            chapters_inodes: tokio::sync::RwLock::default(),
            pages_inodes: tokio::sync::RwLock::default(),
            cover_inodes: tokio::sync::RwLock::default()
        })
    }

    async fn make_next_ino(&self) -> u64 {
        let mut next_ino = self.next_ino.lock().await;
        debug!("generated ino: {}", *next_ino);
        let ret = *next_ino;
        *next_ino += 1u64;
        ret
    }

    async fn new_node(&self, ino: u64, entry: fs::entry::Entry) {
        debug!("writing entry \"{}\" at ino: {}", entry.variant(), ino);
        self.entries.write().await.insert(ino, fs::entry::Inode(entry, fs::entry::Attributes::new(ino, self.uid.clone(), self.gid.clone())));
    }

    pub async fn log_in<L, P>(&self, login: L, password: P) -> Result<api::MangaDexSession, api::LogInError>
        where
        L: Into<std::borrow::Cow<'static, str>>,
        P: Into<std::borrow::Cow<'static, str>> {
        self.api.write().await.log_in(login, password).await.map(std::clone::Clone::clone)
    }

    pub async fn log_out(&self) -> Result<(), api::LogOutError> {
        self.api.write().await.log_out().await
    }

    pub async fn get_or_fetch_manga(&self, id: u64, languages: Vec<String>) -> Result<GetOrFetchRef<fs::entry::Manga>, api::GetMangaError> {
        match self.manga.write().await.entry(id) {
            std::collections::hash_map::Entry::Occupied(occupied) => Ok(GetOrFetchRef::Cached(std::sync::Arc::downgrade(occupied.get()))),
            std::collections::hash_map::Entry::Vacant(vacant) => match self.api.read().await.get_manga(id).await {
                Ok(manga_api) => {
                    let manga = std::sync::Arc::new(fs::entry::Manga::new(id, manga_api));

                    let manga_ino = self.make_next_ino().await;
                    
                    let mut directory = fs::entry::Directory::new(1u64);
                    for chapter in &manga.chapters {
                        if languages.iter().find(|&lang| lang == &chapter.lang_code).is_some() {
                            let chapter_ino: u64 = self.make_next_ino().await;
                            directory.children.insert(chapter.to_string().into(), (chapter_ino, false));
                            self.new_node(chapter_ino, fs::entry::Entry::ChapterNotFetched(chapter.id)).await;
                            self.chapters_inodes.write().await.insert(chapter.id, chapter_ino);
                        }
                    }

                    self.manga_inodes.write().await.insert(manga.id, manga_ino);

                    if let Some(url) = &manga.cover {
                        let cover_ino: u64 = self.make_next_ino().await;
                        directory.children.insert(url
                            .path_segments()
                            .and_then(|split| split.last())
                            .and_then(|last| std::path::Path::new(last).extension())
                            .map(|extension| {
                                let mut cover = std::path::PathBuf::from("cover");
                                cover.set_extension(extension);
                                cover
                            })
                            .unwrap(), (cover_ino, true));
                        self.cover_inodes.write().await.insert(url.clone(), cover_ino);

                        debug!("fetching cover from {}", url);
                        self.get_or_fetch_cover(id, &url).await.ok();
                    }

                    let manga_ref = std::sync::Arc::downgrade(&manga);
                    self.new_node(manga_ino, fs::entry::Entry::Manga(manga_ref, directory)).await;
                    
                    if let Some(fs::entry::Inode(fs::entry::Entry::Root(directory), _)) = self.entries.write().await.get_mut(&1u64) {
                        directory.children.insert(manga.to_string().into(), (manga_ino, false));

                        self.server.lock().await.notify_inval_inode(1u64, 0i64, 0i64).await.ok();

                        Ok(GetOrFetchRef::Fetched(std::sync::Arc::downgrade(vacant.insert(manga))))
                    }
                    else { panic!("root directory is gone?"); }
                },
                Err(error) => Err(error)
            }
        }
    }

    pub async fn get_or_fetch_chapter(self: &std::sync::Arc<Context>, id: u64) -> Result<GetOrFetchRef<fs::entry::Chapter>, api::GetMangaError> {
        match self.chapters.write().await.entry(id) {
            std::collections::hash_map::Entry::Occupied(occupied) => Ok(GetOrFetchRef::Cached(std::sync::Arc::downgrade(occupied.get()))),
            std::collections::hash_map::Entry::Vacant(vacant) => match self.api.read().await.get_chapter(id).await {
                Ok(chapter_api) => {
                    let chapter = std::sync::Arc::new(fs::entry::Chapter::new(id, chapter_api));

                    match self.manga_inodes.read().await.get(&chapter.manga_id).cloned() {
                        Some(manga_ino) => {
                            let chapters_inodes_read_lock = self.chapters_inodes.read().await;

                            match chapters_inodes_read_lock.get(&chapter.id).cloned() {
                                Some(chapter_ino) => {
                                    drop(chapters_inodes_read_lock);

                                    let entries_read_lock =  self.entries.read().await;
                                    match entries_read_lock.get(&chapter_ino) {
                                        Some(fs::entry::Inode(entry, _)) => {
                                            if let fs::entry::Entry::ChapterNotFetched(_) = entry {
                                                drop(entries_read_lock);
                                                debug!("reusing chapter inode: {}", chapter_ino);

                                                let mut directory = fs::entry::Directory::new(1u64);
                                                match &chapter.pages {
                                                    fs::entry::ChapterPages::Hosted(hosted) => {
                                                        let mut tasks = Vec::with_capacity(hosted.pages.len());

                                                        for (index, page) in hosted.pages.iter().enumerate() {
                                                            let url = hosted.url.join(page).unwrap();

                                                            let page_ino: u64 = self.make_next_ino().await;
                                                            directory.children.insert(page.into(), (page_ino, true));
                                                            self.pages_inodes.write().await.insert(url.clone(), page_ino);

                                                            debug!("fetching page {}/{} for chapter {}", index + 1, hosted.pages.len(), id);

                                                            let self_ = self.clone();
                                                            let chapter_id = chapter.id;
                                                            
                                                            tasks.push(tokio::spawn(async move {
                                                                self_.get_or_fetch_page(chapter_id, &url).await.ok();
                                                            }));
                                                        }

                                                        for (index, task) in tasks.into_iter().enumerate() {
                                                            debug!("awaiting page {}/{} for chapter {}", index + 1, hosted.pages.len(), id);
                                                            task.await.ok();
                                                        }
                                                    },
                                                    fs::entry::ChapterPages::External(external) => {
                                                        let external_ino: u64 = self.make_next_ino().await;
                                                        directory.children.insert("external.html".into(), (external_ino, true));
                                    
                                                        let file = {
                                                            let content = format!(
                                                                r#"<!DOCTYPE HTML>
                                                                <html>
                                                                <head>
                                                                <meta http-equiv="refresh" content="0; url={}" />
                                                                </head>
                                                                <body>
                                                                </body>
                                                                </html>"#,
                                                                external.url.to_string()
                                                            );
                                                    
                                                            content.into_bytes()
                                                        };

                                                        self.new_node(external_ino, fs::entry::Entry::External(file)).await;
                                                        self.pages_inodes.write().await.insert(external.url.clone(), external_ino);
                                                    }
                                                };

                                                let chapter_ref = std::sync::Arc::downgrade(&chapter);
                                                self.new_node(chapter_ino, fs::entry::Entry::Chapter(chapter_ref, directory)).await;
                                                self.chapters_inodes.write().await.insert(chapter.id, chapter_ino);
                                                self.server.lock().await.notify_inval_inode(chapter_ino, 0i64, 0i64).await.ok();
                
                                                Ok(GetOrFetchRef::Fetched(std::sync::Arc::downgrade(vacant.insert(chapter))))
                                            }
                                            else {
                                                panic!("cached chapter inode is not a ChapterNotFetched inode?");
                                            }
                                        },
                                        None => panic!("cached chapter inode is invalid?")
                                    }
                                },
                                None => {
                                    drop(chapters_inodes_read_lock);
                                    debug!("no chapter inode, creating a new one");

                                    let chapter_ino = self.make_next_ino().await;
                        
                                    let mut directory = fs::entry::Directory::new(1u64);
                                    match &chapter.pages {
                                        fs::entry::ChapterPages::Hosted(hosted) => {
                                            let mut tasks = Vec::with_capacity(hosted.pages.len());

                                            for (index, page) in hosted.pages.iter().enumerate() {
                                                let url = hosted.url.join(page).unwrap();

                                                let page_ino: u64 = self.make_next_ino().await;
                                                directory.children.insert(page.into(), (page_ino, true));
                                                self.pages_inodes.write().await.insert(url.clone(), page_ino);

                                                debug!("fetching page {}/{} for chapter {}", index + 1, hosted.pages.len(), id);

                                                let self_ = self.clone();
                                                let chapter_id = chapter.id;
                                                
                                                tasks.push(tokio::spawn(async move {
                                                    self_.get_or_fetch_page(chapter_id, &url).await.ok();
                                                }));
                                            }

                                            for (index, task) in tasks.into_iter().enumerate() {
                                                debug!("awaiting page {}/{} for chapter {}", index + 1, hosted.pages.len(), id);
                                                task.await.ok();
                                            }
                                        },
                                        fs::entry::ChapterPages::External(external) => {
                                            let external_ino: u64 = self.make_next_ino().await;
                                            directory.children.insert("external.html".into(), (external_ino, true));
                        
                                            let file = {
                                                let content = format!(
                                                    r#"<!DOCTYPE HTML>
                                                    <html>
                                                    <head>
                                                    <meta http-equiv="refresh" content="0; url={}" />
                                                    </head>
                                                    <body>
                                                    </body>
                                                    </html>"#,
                                                    external.url.to_string()
                                                );
                                        
                                                content.into_bytes()
                                            };
                        
                                            self.new_node(external_ino, fs::entry::Entry::External(file)).await;
                                        }
                                    };

                                    let chapter_ref = std::sync::Arc::downgrade(&chapter);
                                    self.new_node(chapter_ino, fs::entry::Entry::Chapter(chapter_ref, directory)).await;
                                    self.chapters_inodes.write().await.insert(chapter.id, chapter_ino);

                                    if let Some(fs::entry::Inode(fs::entry::Entry::Manga(_, directory), _)) = self.entries.write().await.get_mut(&manga_ino) {
                                        directory.children.insert(chapter.to_string().into(), (manga_ino, true));

                                        self.server.lock().await.notify_inval_inode(manga_ino, 0i64, 0i64).await.ok();

                                        Ok(GetOrFetchRef::Fetched(std::sync::Arc::downgrade(vacant.insert(chapter))))
                                    }
                                    else { panic!("manga inode not valid?"); }
                                }
                            }
                        },
                        None => panic!("manga inode not saved in inodes map?")
                    }
                },
                Err(error) => Err(error)
            }
        }
    }

    pub async fn get_or_fetch_page(&self, chapter_id: u64, url: &reqwest::Url) -> Result<GetOrFetchRef<fs::entry::Page>, api::GetPageError> {
        match self.pages.write().await.entry(url.clone()) {
            std::collections::hash_map::Entry::Occupied(occupied) => Ok(GetOrFetchRef::Cached(std::sync::Arc::downgrade(occupied.get()))),
            std::collections::hash_map::Entry::Vacant(vacant) => match self.api.read().await.get_page(chapter_id, &url).await {
                Ok(page_api) => {
                    let page = std::sync::Arc::new(fs::entry::Page(page_api.data));

                    match self.chapters_inodes.read().await.get(&chapter_id).cloned() {
                        Some(chapter_ino) => {
                            let pages_inodes_read_lock = self.pages_inodes.read().await;

                            match pages_inodes_read_lock.get(&url).cloned() {
                                Some(page_ino) => {
                                    drop(pages_inodes_read_lock);
                                    debug!("reusing page inode: {}", page_ino);

                                    let page_ref = std::sync::Arc::downgrade(&page);
                                    
                                    self.new_node(page_ino, fs::entry::Entry::Page(page_ref)).await;
                                    self.server.lock().await.notify_inval_inode(chapter_ino, 0i64, 0i64).await.ok();

                                    Ok(GetOrFetchRef::Fetched(std::sync::Arc::downgrade(vacant.insert(page))))
                                },
                                None => {
                                    let page_ino = self.make_next_ino().await;
                                    debug!("creating page inode: {}", page_ino);

                                    let page_ref = std::sync::Arc::downgrade(&page);
                                    
                                    self.new_node(page_ino, fs::entry::Entry::Page(page_ref)).await;
                                    self.server.lock().await.notify_inval_inode(chapter_ino, 0i64, 0i64).await.ok();
    
                                    Ok(GetOrFetchRef::Fetched(std::sync::Arc::downgrade(vacant.insert(page))))
                                }
                            }
                        },
                        None => panic!("chapter inode not saved in inodes map?")
                    }
                },
                Err(error) => Err(error)
            }
        }
    }

    pub async fn get_or_fetch_cover(&self, manga_id: u64, url: &reqwest::Url) -> Result<GetOrFetchRef<fs::entry::Cover>, api::GetCoverError> {
        match self.covers.write().await.entry(url.clone()) {
            std::collections::hash_map::Entry::Occupied(occupied) => Ok(GetOrFetchRef::Cached(std::sync::Arc::downgrade(occupied.get()))),
            std::collections::hash_map::Entry::Vacant(vacant) => match self.api.read().await.get_cover(&url).await {
                Ok(cover_api) => {
                    let cover = std::sync::Arc::new(fs::entry::Cover(cover_api.0));

                    match self.manga_inodes.read().await.get(&manga_id).cloned() {
                        Some(manga_ino) => {
                            let cover_inodes_read_lock = self.cover_inodes.read().await;

                            match cover_inodes_read_lock.get(&url).cloned() {
                                Some(cover_ino) => {
                                    drop(cover_inodes_read_lock);
                                    debug!("reusing cover inode: {}", cover_ino);

                                    let cover_ref = std::sync::Arc::downgrade(&cover);
                                    
                                    self.new_node(cover_ino, fs::entry::Entry::Cover(cover_ref)).await;
                                    self.server.lock().await.notify_inval_inode(manga_ino, 0i64, 0i64).await.ok();

                                    Ok(GetOrFetchRef::Fetched(std::sync::Arc::downgrade(vacant.insert(cover))))
                                },
                                None => {
                                    let cover_ino = self.make_next_ino().await;
                                    debug!("creating cover inode: {}", cover_ino);

                                    let cover_ref = std::sync::Arc::downgrade(&cover);
                                    
                                    self.new_node(cover_ino, fs::entry::Entry::Cover(cover_ref)).await;
                                    self.server.lock().await.notify_inval_inode(manga_ino, 0i64, 0i64).await.ok();
    
                                    Ok(GetOrFetchRef::Fetched(std::sync::Arc::downgrade(vacant.insert(cover))))
                                }
                            }
                        },
                        None => panic!("manga inode not saved in inodes map?")
                    }
                },
                Err(error) => Err(error)
            }
        }
    }

    pub async fn search(&self, params: &api::SearchParams) -> Result<Vec<api::SearchEntry>, api::APIError> {
        self.api.read().await.search(params).await
    }

    pub async fn mdlist(&self, params: &api::MDListParams) -> Result<Vec<api::MDListEntry>, api::MDListError> {
        self.api.read().await.mdlist(params).await
    }

    pub async fn follow(&self, id: u64, status: &api::MDListStatus) -> Result<(), api::APIError> {
        self.api.read().await.follow(id, status).await
    }

    pub async fn unfollow(&self, id: u64) -> Result<(), api::APIError> {
        self.api.read().await.unfollow(id).await
    }


    pub async fn mark_chapter_read(&self, id: u64) -> Result<(), api::APIError> {
        self.api.read().await.mark_chapter_read(id).await
    }

    pub async fn mark_chapter_unread(&self, id: u64) -> Result<(), api::APIError> {
        self.api.read().await.mark_chapter_unread(id).await
    }

    pub async fn follows(&self) -> Result<Vec<api::FollowsEntry>, api::APIError> {
        self.api.read().await.follows().await
    }
}
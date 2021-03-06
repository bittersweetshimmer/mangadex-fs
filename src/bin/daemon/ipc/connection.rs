use mangadex_fs::ipc;
use mangadex_fs::api;

use mangadex_fs::ipc::{IpcSend, IpcTryReceive};

pub struct Connection {
    stream: tokio::net::UnixStream,
    context: std::sync::Arc<mangadex_fs::Context>,
    kill: tokio::sync::mpsc::Sender<()>
}

impl Connection {
    pub fn new(
        stream: tokio::net::UnixStream,
        context: std::sync::Arc<mangadex_fs::Context>,
        kill: tokio::sync::mpsc::Sender<()>
    ) -> Connection {
        Connection { stream, context, kill }
    }

    pub async fn read_eval_loop(&mut self) -> std::io::Result<()> {
        loop {
            return match ipc::Command::ipc_try_receive(&mut self.stream).await? {
                Some(command) => {
                    let response = match command {
                        ipc::Command::EndConnection => return Ok(()),
                        ipc::Command::Kill => self.kill().await?,
                        ipc::Command::LogIn(username, password) => self.log_in(username, password).await?,
                        ipc::Command::LogOut => self.log_out().await?,
                        ipc::Command::AddManga(id, languages) => self.add_manga(id, languages).await?,
                        ipc::Command::Search(params) => self.search(&params).await?,
                        ipc::Command::MDList(params) => self.mdlist(&params).await?,
                        ipc::Command::FollowManga(id, status) => self.follow(id, &status).await?,
                        ipc::Command::UnfollowManga(id) => self.unfollow(id).await?,
                        ipc::Command::MarkChapterRead(id) => self.mark_chapter_read(id).await?,
                        ipc::Command::MarkChapterUnread(id) => self.mark_chapter_unread(id).await?,
                        ipc::Command::Follows => self.follows().await?
                    };

                    response.ipc_send(&mut self.stream).await?;

                    Ok(())
                },
                None => Ok(())
            }
        }
    }

    pub async fn kill(&mut self) -> std::io::Result<ipc::Response> {
        self.kill.send(()).await.expect("MikuDex");

        Ok(ipc::Response::Kill)
    }

    pub async fn log_in(&mut self, username: String, password: String) -> std::io::Result<ipc::Response> {
        Ok(match self.context.log_in(username.clone(), password).await {
            Ok(session) => {
                info!("logged in successfully as {}", username);
                debug!("session id: {}", session.id);

                ipc::Response::LogIn(Ok(session.clone()))
            },
            Err(error) => {
                warn!("log in error: {:?}", error);

                match error {
                    api::LogInError::Response(body) => ipc::Response::LogIn(Err(String::from("MangaDex response: ") + &body)),
                    _ => ipc::Response::LogIn(Err("request error".into()))
                }
            }
        })
    }

    pub async fn log_out(&mut self) -> std::io::Result<ipc::Response> {
        Ok(match self.context.log_out().await {
            Ok(_) => {
                info!("logged out successfully");

                ipc::Response::LogOut(Ok(()))
            },
            Err(error) => {
                warn!("log out error: {:?}", error);

                ipc::Response::LogOut(Err("request error".into()))
            }
        })
    }

    pub async fn add_manga(&mut self, manga_id: u64, languages: Vec<String>) -> std::io::Result<ipc::Response> {
        Ok(match self.context.get_or_fetch_manga(manga_id, languages).await {
            Ok(mangadex_fs::GetOrFetch::Cached(manga_ref)) => match manga_ref.upgrade() {
                Some(manga) => {
                    info!("cached manga {}: {}", manga_id, manga.to_string());

                    ipc::Response::AddManga(Ok(manga.title.clone()))
                },
                None => {
                    warn!("cached manga {}: pointer dropped?", manga_id);

                    ipc::Response::AddManga(Err("pointer dropped".into()))
                }
            },
            Ok(mangadex_fs::GetOrFetch::Fetched(manga_ref)) => match manga_ref.upgrade() {
                Some(manga) => {
                    info!("fetched manga {}: {}", manga_id, manga.to_string());

                    ipc::Response::AddManga(Ok(manga.title.clone()))
                },
                None => {
                    warn!("fetched manga {}: pointer dropped?", manga_id);

                    ipc::Response::AddManga(Err("pointer dropped".into()))
                }
            },
            Err(error) => {
                warn!("add manga request error: {}", error);

                ipc::Response::AddManga(Err("request error".into()))
            }
        })
    }

    pub async fn search(&mut self, params: &api::SearchParams) -> std::io::Result<ipc::Response> {
        Ok(match self.context.search(params).await {
            Ok(results) => {
                info!("found {} results for search query {:?}", results.len(), params);

                ipc::Response::Search(Ok(results))
            },
            Err(error) => {
                warn!("search error: {:?}", error);
                
                match error {
                    api::APIError::Request(_) => ipc::Response::Search(Err("request error".into())),
                    api::APIError::NotLoggedIn => ipc::Response::Search(Err("you need to be logged in to use this command".into()))
                }
            }
        })
    }

    pub async fn mdlist(&mut self, params: &api::MDListParams) -> std::io::Result<ipc::Response> {
        Ok(match self.context.mdlist(params).await {
            Ok(results) => {
                info!("found {} results for mdlist {:?}", results.len(), params.id);

                ipc::Response::MDList(Ok(results))
            },
            Err(error) => {
                warn!("mdlist error: {:?}", error);
                
                ipc::Response::Search(Err("request error".into()))
            }
        })
    }

    pub async fn follow(&mut self, id: u64, status: &api::MDListStatus) -> std::io::Result<ipc::Response> {
        Ok(match self.context.follow(id, status).await {
            Ok(_) => {
                ipc::Response::FollowManga(Ok(()))
            },
            Err(error) => {
                warn!("follow error: {:?}", error);
                
                match error {
                    api::APIError::Request(_) => ipc::Response::FollowManga(Err("request error".into())),
                    api::APIError::NotLoggedIn => ipc::Response::FollowManga(Err("you need to be logged in to use this command".into()))
                }
            }
        })
    }

    pub async fn unfollow(&mut self, id: u64) -> std::io::Result<ipc::Response> {
        Ok(match self.context.unfollow(id).await {
            Ok(_) => {
                ipc::Response::UnfollowManga(Ok(()))
            },
            Err(error) => {
                warn!("unfollow error: {:?}", error);
                
                match error {
                    api::APIError::Request(_) => ipc::Response::UnfollowManga(Err("request error".into())),
                    api::APIError::NotLoggedIn => ipc::Response::UnfollowManga(Err("you need to be logged in to use this command".into()))
                }
            }
        })
    }


    pub async fn mark_chapter_read(&mut self, id: u64) -> std::io::Result<ipc::Response> {
        Ok(match self.context.mark_chapter_read(id).await {
            Ok(_) => {
                ipc::Response::MarkChapterRead(Ok(()))
            },
            Err(error) => {
                warn!("mark chapter read error: {:?}", error);
                
                match error {
                    api::APIError::Request(_) => ipc::Response::MarkChapterRead(Err("request error".into())),
                    api::APIError::NotLoggedIn => ipc::Response::MarkChapterRead(Err("you need to be logged in to use this command".into()))
                }
            }
        })
    }

    pub async fn mark_chapter_unread(&mut self, id: u64) -> std::io::Result<ipc::Response> {
        Ok(match self.context.mark_chapter_unread(id).await {
            Ok(_) => {
                ipc::Response::MarkChapterUnread(Ok(()))
            },
            Err(error) => {
                warn!("mark chapter unread error: {:?}", error);
                
                match error {
                    api::APIError::Request(_) => ipc::Response::MarkChapterUnread(Err("request error".into())),
                    api::APIError::NotLoggedIn => ipc::Response::MarkChapterUnread(Err("you need to be logged in to use this command".into()))
                }
            }
        })
    }

    pub async fn follows(&mut self) -> std::io::Result<ipc::Response> {
        Ok(match self.context.follows().await {
            Ok(follows) => {
                info!("found {} entries for follows", follows.len());

                ipc::Response::Follows(Ok(follows))
            },
            Err(error) => {
                warn!("follows error: {:?}", error);
                
                match error {
                    api::APIError::Request(_) => ipc::Response::Follows(Err("request error".into())),
                    api::APIError::NotLoggedIn => ipc::Response::Follows(Err("you need to be logged in to use this command".into()))
                }
            }
        })
    }
}
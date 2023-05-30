use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use config::Config;
mod config;

use response::Response;
mod response;

mod page;

use cache::Cache;
mod cache;

mod content;

pub struct Server {
    listener: TcpListener,
    config: Config,
    cache: Option<Cache>,
    visits: u128,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        let config: Config = confy::load("salient", None).unwrap();
        let listener = TcpListener::bind(&config.address).unwrap();

        let cache: Option<Cache> = if config.caching {
            Some(Cache::new())
        } else {
            None
        };

        let visits = 0u128;

        Server {
            listener,
            config,
            cache,
            visits,
        }
    }

    pub fn run(&mut self) {
        let double_dot_defence = self.config.double_dot_defence;
        for stream in self.listener.incoming() {
            let stream = match stream {
                Ok(stream) => stream,
                Err(_) => continue,
            };

            if self.config.statistics {
                self.visits += 1;
            }

            let cache = self.cache.clone();

            thread::spawn(move || {
                Self::handle_connection(stream, double_dot_defence, cache);
            });
        }
    }

    fn handle_connection(mut stream: TcpStream, double_dot_defence: bool, cache: Option<Cache>) {
        let buf_reader = BufReader::new(&mut stream);
        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        let request_type = http_request[0].split(' ').nth(0).unwrap_or("GET");
        let path = http_request[0].split(' ').nth(1).unwrap_or("/");

        if request_type == "GET" {
            let path = content::format_path(path, &double_dot_defence);
            let response: Response = match cache {
                None => content::page_from_file(&path),
                Some(cache) => content::page_from_cache(&cache, &path),
            };
            let _ = stream.write_all(response.bytes());
        } else if request_type == "POST" {
            match path {
                _ => {}
            }
        }
    }
}

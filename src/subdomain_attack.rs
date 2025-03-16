use std::{fs::File, io::BufReader, io::Lines};
use std::sync::{Arc,Mutex};

use reqwest::Client;


 pub async fn start(
    buffer_iterator: Arc<Mutex<Lines<BufReader<File>>>>,
    url_arc: Arc<String>,
    client: Arc<Client>,
    threads: i32,
    work_per_thread: u64){
    
    let mut handles = vec![];
    for _ in 0..threads {
        let buff_clone = Arc::clone(&buffer_iterator);
        let url = Arc::clone(&url_arc);
        let client_clone = Arc::clone(&client);
        let thread = tokio::task::spawn( async move {    
            for _ in 0..work_per_thread{
                let line = buff_clone.lock().unwrap().next().unwrap().unwrap();
                let line_clone = line.clone();
                drop(line);
                

                let split: Vec<&str> = url.split("://").collect();
                let subdomain = {
                    let subdomain = format!("{}://{}.{}",split[0],line_clone,split[1]);
                    subdomain
                };

                let host = format!("{}.{}",line_clone,split[1]).replace("/","");

                let response = client_clone.get(&subdomain).header("Host", host).send().await;
                
                if let Ok(resp) = response {
                    if resp.status() != 404 {
                        println!("URL: {}  -- STATUS CODE {}",subdomain,resp.status());
                    }
                }
            }
        });
        handles.push(thread);
    }

    for handle in handles {
        handle.await.unwrap();
    }

 }

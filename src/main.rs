use std::env;
use reqwest;
use tokio;
use std::fs::File;
use std::io::{BufReader,BufRead};
use std::sync::{Arc, Mutex};

mod subdomain_attack;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut url: String = "".to_string();  

    for i in 0..args.len(){
        if args[i] == "-u" {
            url = args[i+1].clone();
        }
    }
    if url.is_empty() {
        println!("not -u parameter found refering to url");
        std::process::exit(1);
    }
    if !(url.starts_with("http")){
        println!("url should contain http or https");
        std::process::exit(1);
    }



    let mut wordlist: String = "".to_string();
    for i in 0..args.len() {
        if args[i] == "-w" {
            wordlist = args[i+1].clone();
        }
    }
    if wordlist.is_empty(){
        println!("not -w parameter found refering to the wordlist");
        std::process::exit(1);
    }
        
    
    let mut threads: i32 = 1;

    for i in 0..args.len() {
        if args[i] == "-t" {
            threads = args[i+1].parse().unwrap();
        }
    }

    let file = File::open(&wordlist).unwrap();
    let buffer = BufReader::new(file);

    let total_lines: u64 = {
        let mut i = 0;
        for _ in buffer.lines() {
            i=i+1;
        }
        i
    };

    let work_per_thread = total_lines / threads as u64;

    let file = File::open(wordlist).unwrap();
    let buffer = BufReader::new(file);
    let buffer_iterator = Arc::new(Mutex::new(buffer.lines())); 
    let mut handles = vec![];
    let url_arc = Arc::new(url.clone());
    let client = Arc::new(reqwest::Client::new());


    for i in 0..args.len() {
        if args[i] == "--subdomain" {
            subdomain_attack::start(buffer_iterator,url_arc,client,threads,work_per_thread).await;
            std::process::exit(0);
        }
    }

    

    for _ in 0..threads {
        let buff_clone = Arc::clone(&buffer_iterator);
        let url = Arc::clone(&url_arc);
        let client_clone = Arc::clone(&client);
        let thread = tokio::task::spawn( async move {    
            for _ in 0..work_per_thread{
                let line = buff_clone.lock().unwrap().next().unwrap().unwrap();
                let line_clone = line.clone();
                drop(line);

                let response = client_clone.get(&url.replace("FUZZ",&line_clone)).send().await;

                if let Ok(resp) = response {
                    if resp.status() != 404 {
                        println!("URL: {}  -- STATUS CODE {}",&url.replace("FUZZ", &line_clone),resp.status());
                    }
                }
            }
        });
        handles.push(thread);
    }

    for handle in handles {
        handle.await.unwrap();
    }
    Ok(())
}

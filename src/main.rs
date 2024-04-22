use dialoguer::FuzzySelect;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};
use std::cmp::min;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;
use std::{env, io};
use tokio::runtime::Runtime;
use tokio::task;

#[derive(Debug)]
struct SearchResult {
    author: String,
    title: String,
    year: String,
    pages: String,
    file_format: String,
    dl_page: String,
}

static ERR_BACKEND_CHANGED: &str =
    "Backend change detected. Please check the GitHub for an updated client. Exiting...";

fn tr_to_search_result(tr: ElementRef) -> SearchResult {
    let a = Selector::parse("a").expect(ERR_BACKEND_CHANGED);
    let x: Vec<_> = (0..10)
        .map(|x| tr.child_elements().nth(x).expect(ERR_BACKEND_CHANGED))
        .collect();

    SearchResult {
        author: x[1]
            .select(&a)
            .next()
            .expect(ERR_BACKEND_CHANGED)
            .inner_html()
            .trim()
            .to_string(),
        title: x[2]
            .select(&a)
            .next()
            .expect(ERR_BACKEND_CHANGED)
            .text()
            .next()
            .expect(ERR_BACKEND_CHANGED)
            .trim()
            .to_string(),
        year: x[4].inner_html().trim().to_string(),
        pages: x[5].inner_html().trim().to_string(),
        file_format: x[8].inner_html().trim().to_string(),
        dl_page: x[9]
            .select(&a)
            .next()
            .expect(ERR_BACKEND_CHANGED)
            .attr("href")
            .expect(ERR_BACKEND_CHANGED)
            .trim()
            .to_string(),
    }
}

fn libg_search(search_term: &str) -> Vec<SearchResult> {
    // Pagination
    let mut page = 1;
    let mut all_results: Vec<SearchResult> = Vec::new();

    // Domain fallbacks in case of server downtime
    let domains = ["libgen.rs", "libgen.is", "libgen.st"];
    let mut domain_choice = 0;

    loop {
        let url = format!(
            "https://{}/search.php?res=100&req={}&page={}",
            domains[domain_choice], search_term, page
        );
        let response = reqwest::blocking::get(url)
            .expect("Error in certificate chain.")
            .error_for_status();
        let response_text: String;

        // Fall back on alternative domain if the current one is down
        match response {
            Ok(response) => {
                response_text = response.text().expect(ERR_BACKEND_CHANGED);
            }
            Err(..) => {
                if domain_choice == domains.len() - 1 {
                    println!("All backend servers are down. Check the GitHub for an updated client. Exiting...");
                    exit(1);
                } else {
                    domain_choice += 1;
                    continue;
                }
            }
        }

        let document = Html::parse_document(&response_text);

        let toplevel_selector = Selector::parse(".c > tbody").expect(ERR_BACKEND_CHANGED);
        let search_table_result: Option<ElementRef<'_>> =
            document.select(&toplevel_selector).next();

        // Check if search results are found
        match search_table_result {
            Some(search_table) => {
                let select_rows = Selector::parse("tr").expect(ERR_BACKEND_CHANGED);

                if search_table.select(&select_rows).count() == 1 {
                    break;
                }

                let row_iterator = search_table.select(&select_rows).skip(1); //Note: skip(1) skips the table header

                let new_row_structs: Vec<_> = row_iterator.map(tr_to_search_result).collect();
                // Concatenate to all_results
                all_results.extend(new_row_structs);
            }
            None => {
                return Vec::new();
            }
        }

        page += 1;
    }

    all_results
}

fn libg_get_download(dl_page: &str) -> String {
    let response = reqwest::blocking::get(dl_page)
        .expect(ERR_BACKEND_CHANGED)
        .error_for_status()
        .expect(ERR_BACKEND_CHANGED)
        .text()
        .expect(ERR_BACKEND_CHANGED);
    let document = Html::parse_document(&response);
    let toplevel_selector = Selector::parse("#download").expect(ERR_BACKEND_CHANGED);
    let toplevel_div = document
        .select(&toplevel_selector)
        .next()
        .expect(ERR_BACKEND_CHANGED);
    return toplevel_div
        .descendent_elements()
        .nth(2)
        .expect(ERR_BACKEND_CHANGED)
        .attr("href")
        .expect(ERR_BACKEND_CHANGED)
        .to_string();
}

fn help() {
    println!("Tool to download books from supported archive sites.\n");
    println!("Usage: libgen-rs [SEARCH TERM]");
    println!("libgen-rs [OPTIONS]");
    println!("libgen-rs\n");
    println!("Options:");
    println!("-h, --help      Display this help message");
}

fn read_string() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Cannot read user input.");
    input
}

fn parse_args() -> String {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            print!("Enter search term: ");
            io::stdout().flush().expect("Failed to flush stdout");

            read_string()
        }
        _ => {
            // If command is -help, print the help
            if args[1].to_lowercase() == "--help" || args[1].to_lowercase() == "-h" {
                help();
                exit(0);
            } else {
                // Concatenate the search term
                let mut search_term = String::new();
                search_term.push_str(&args[1]);
                for item in args.iter().skip(2) {
                    search_term.push(' ');
                    search_term.push_str(item);
                }

                search_term
            }
        }
    }
}

// Inspired by https://gist.github.com/giuliano-macedo/4d11d6b3bb003dba3a1b53f43d81b30d
async fn download_search_result(
    client: &Client,
    search_result: &SearchResult,
    directory: PathBuf,
) -> Result<(), String> {
    // Extract the download link
    let dl_page_clone = search_result.dl_page.clone();

    // Extract the download link asynchronously
    let url = &task::spawn_blocking(move || libg_get_download(&dl_page_clone))
        .await
        .expect("Error while fetching download link");

    // Reqwest setup
    let res = client
        .get(url)
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", url)))?;
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", url))?;

    // Indicatif setup
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .unwrap()
        .progress_chars("#>-"));
    pb.set_message(format!("Downloading {}", url));

    // Combine directory and file name
    let path = directory.join(sanitize_filename(&format!(
        "{}.{}",
        search_result.title, search_result.file_format
    )));
    let path_string = path.to_str().unwrap();

    // download chunks
    let mut file =
        File::create(path_string).or(Err(format!("Failed to create file '{}'", path_string)))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err("Error while downloading file".to_string()))?;
        file.write_all(&chunk)
            .or(Err("Error while writing to file".to_string()))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(format!("Downloaded {} to {}", url, path_string));
    Ok(())
}

fn sanitize_filename(filename: &str) -> String {
    // Sanitize filename according to this list https://stackoverflow.com/a/31976060
    let forbidden_chars_regex = Regex::new(r#"[<>:"/\\|?*\x00-\x1F]"#).unwrap(); // Forbidden characters on Windows and control characters
    let reserved_names_regex =
        Regex::new(r#"(?i)^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\..*)?$"#).unwrap(); // Reserved names on Windows

    let mut sanitized = forbidden_chars_regex.replace_all(filename, "_").to_string();

    // Check for reserved names and append underscores if necessary
    if reserved_names_regex.is_match(&sanitized) {
        sanitized.push('_');
    }

    sanitized = sanitized
        .trim_end_matches(|c| c == '.' || c == ' ')
        .to_string();

    sanitized
}

fn stringify_search_results(results: &[SearchResult]) -> Vec<String> {
    // Convert into string array
    let mut result_options = Vec::new();

    for (i, x) in results.iter().enumerate() {
        // Account for the fact that some books differentiate pages and pages with content
        let page_count = if x.pages.is_empty() || x.pages.starts_with('0') {
            "".to_string()
        } else if x.pages.contains('[') {
            x.pages
                .split('[')
                .collect::<Vec<&str>>()
                .get(1)
                .unwrap()
                .replace(']', "")
                .to_string()
        } else {
            x.pages.to_string()
        };

        let page_info =
            // Correct pluralization
            if !page_count.is_empty() {
                if page_count.len() == 1 && page_count.starts_with('1') {
                    format!("{} page - ", page_count)
                } else {
                    format!("{} pages - ", page_count)
                }
            } else { String::new() };

        let author = if x.author.is_empty() {
            "Unknown author"
        } else {
            &x.author
        };

        result_options.push(format!(
            "{}. \"{}\" by {}, {} ({}{})",
            i + 1,
            x.title,
            author,
            x.year,
            page_info,
            x.file_format
        ));
    }
    result_options
}

fn main() {
    // Extract search term
    let search_term = parse_args();

    // Since an empty string has the length 2, we need to check against 5 is we want 3 characters
    if search_term.len() < 5 {
        println!("Search term must be at least 3 characters long.");
        exit(1);
    }

    println!("Searching for: {}", search_term);

    let results = &libg_search(&search_term);

    if results.is_empty() {
        println!("No results found for search term: {}", search_term);
        exit(1);
    } else {
        let result_options = stringify_search_results(results);

        // Fuzzy select the desired result
        let selected = FuzzySelect::new()
            .with_prompt(format!("Select out of {} books:", results.len()))
            .items(&result_options)
            .default(0)
            .interact()
            .expect("Dialoguer Issue. Please use a different terminal emulator.");

        let selected_result = &results[selected];

        // Get current directory
        let current_dir = env::current_dir()
            .expect("Please run the program in a directory where you have write permissions.");

        // Run the download_file function within the Tokio runtime
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            let client = Client::new();

            if let Err(err) = download_search_result(&client, selected_result, current_dir).await {
                eprintln!("Error downloading file: {}", err);
            } else {
                println!("File successfully downloaded!");
            }
        });
    }
}

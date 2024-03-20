//use fuzzy_select::FuzzySelect;
use scraper::{ElementRef, Html, Selector};

#[derive(Debug)]
struct SearchResult {
    id: String,
    author: String,
    title: String,
    publisher: String,
    year: String,
    pages: String,
    language: String,
    file_size: String,
    file_format: String,
    dl_link: String,
}

fn tr_to_searchresult (tr:ElementRef) -> SearchResult {
    return SearchResult {
        id:          tr.child_elements().nth(0).unwrap().inner_html().to_string(),
        author:      tr.child_elements().nth(1).unwrap().inner_html().to_string(), //TODO
        title:       tr.child_elements().nth(2).unwrap().inner_html().to_string(), //TODO
        publisher:   tr.child_elements().nth(3).unwrap().inner_html().to_string(),
        year:        tr.child_elements().nth(4).unwrap().inner_html().to_string(),
        pages:       tr.child_elements().nth(5).unwrap().inner_html().to_string(),
        language:    tr.child_elements().nth(6).unwrap().inner_html().to_string(),
        file_size:   tr.child_elements().nth(7).unwrap().inner_html().to_string(),
        file_format: tr.child_elements().nth(8).unwrap().inner_html().to_string(),
        dl_link:     tr.child_elements().nth(9).unwrap().inner_html().to_string(), //TODO
    };
}

fn libgsearch (searchterm:&str) -> Vec<SearchResult> {
    let base = "https://libgen.rs/search.php?res=100&req=";
    let url = format!("{}{}", base, searchterm);
    let response = reqwest::blocking::get(url).unwrap().error_for_status().unwrap().text().unwrap();

    let document = Html::parse_document(&response);
    let toplevel_selector = Selector::parse(".c > tbody").unwrap();
    let search_table = document.select(&toplevel_selector).next().unwrap();

    let select_rows = Selector::parse("tr").unwrap();
    let row_iterator = search_table.select(&select_rows).skip(1); //Note: skip(1) skips the table header
    let rowstructs = row_iterator.map(tr_to_searchresult).collect();
    return rowstructs;
}

fn main() {
    let x = libgsearch("harry");
    println!("{:#?}", x);

    // Fuzzy_select How To
    // let options = vec!["vanilla", "strawberry", "chocolate"];
    // let selected = FuzzySelect::new()
    //     .with_prompt("What's your favorite flavor of ice cream?")
    //     .with_options(options)
    //     .select();
    // println!("\nYour favorite ice cream flavor is {:?}\n", selected);
}

//use fuzzy_select::FuzzySelect;
use unhtml::FromHtml;


#[derive(FromHtml)]
struct SearchTable {
    //path e.g. .c > tbody:nth-child(1) > tr:nth-child(1)
    #[html(attr="valign")]
    entries: Vec<SearchResult>,
}

// <tr valign=top bgcolor=#C6DEFF><td>58419</td>
#[derive(FromHtml)]
struct SearchResult {
    #[html(selector = "tr:nth-child(1)", attr = "inner")]
    id: i32,
    #[html(selector = "tr:nth-child(2)", attr = "inner")]
    //TODO This no workey because stupid hrefs
    author: String,
    #[html(selector = "tr:nth-child(3)", attr = "inner")]
    //TODO Again no workey....
    title: String,
    #[html(selector = "tr:nth-child(4)", attr = "inner")]
    publisher: String,
    #[html(selector = "tr:nth-child(5)", attr = "inner")]
    year: i32,
    #[html(selector = "tr:nth-child(6)", attr = "inner")]
    pages: i32,
    #[html(selector = "tr:nth-child(7)", attr = "inner")]
    language: String,
    #[html(selector = "tr:nth-child(8)", attr = "inner")]
    file_size: String,
    #[html(selector = "tr:nth-child(9)", attr = "inner")]
    file_format: String,
    #[html(selector = "tr:nth-child(10)", attr = "inner")]
    //TODO deal with lé href again
    dl_link: String,
}

fn libgsearch (endpoint:&str) -> Result<String, reqwest::Error> {
    // enpoint := what comes after the / of the url. e.g.: https://libgen.rs/enpoint has the endpoint /enpoint
    // TODO Make function actually fail over to alternatives: libgen.is or libgen.st at Error match or non-200 status
    let base = "https://libgen.rs/search.php?res=100&req=";
    let url = format!("{}{}", base, endpoint);
    let response = reqwest::blocking::get(url)?.error_for_status();
    return Ok(response?.text()?);
}

fn main() {
    let x = libgsearch("harry");
    let y = match x {
        Ok(y) => y,
        Err(_y) => todo!(),
    };

    let r1 = SearchTable::from_html(&y);

    println!("{}", y);

    // Fuzzy_select How To
    // let options = vec!["vanilla", "strawberry", "chocolate"];
    // let selected = FuzzySelect::new()
    //     .with_prompt("What's your favorite flavor of ice cream?")
    //     .with_options(options)
    //     .select();
    // println!("\nYour favorite ice cream flavor is {:?}\n", selected);
}

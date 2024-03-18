//use fuzzy_select::FuzzySelect;

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
        Err(y) => todo!(),
    };

    println!("{}", y);

    // Fuzzy_select How To
    // let options = vec!["vanilla", "strawberry", "chocolate"];
    // let selected = FuzzySelect::new()
    //     .with_prompt("What's your favorite flavor of ice cream?")
    //     .with_options(options)
    //     .select();
    // println!("\nYour favorite ice cream flavor is {:?}\n", selected);
}

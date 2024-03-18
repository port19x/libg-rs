use reqwest::blocking::Client;
//use fuzzy_select::FuzzySelect;

fn main() {
    // Reqwest how to
    let request = Client::new()
        .get("https://port19.xyz");

    let result = request.send();

    match result {
        Ok(response) => println!("{}", response.status()),
        Err(_err) => todo!(),
    }

    //fuzzy_select how to
    // let options = vec!["vanilla", "strawberry", "chocolate"];
    // let selected = FuzzySelect::new()
    //     .with_prompt("What's your favorite flavor of ice cream?")
    //     .with_options(options)
    //     .select();
    // println!("\nYour favorite ice cream flavor is {:?}\n", selected);
}

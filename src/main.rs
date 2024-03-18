use reqwest::blocking::Client;

fn main() {
    let request = Client::new()
        .get("https://port19.xyz");

    let result = request.send();

    match result {
        Ok(response) => println!("{}", response.status()),
        Err(_err) => todo!(),
    }
}

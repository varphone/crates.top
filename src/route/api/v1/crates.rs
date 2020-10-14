use rocket::Route;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

pub fn routes() -> Vec<Route> {
    routes![index]
}

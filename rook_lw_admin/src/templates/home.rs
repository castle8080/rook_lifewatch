use maud::{html, Markup};

pub fn home_page() -> Markup {
    html! {
        html {
            head {
                title { "Welcome to Rook LifeWatch Admin" }
            }
            body {
                h1 { "Welcome!" }
                p { "This is the home page rendered with Maud." }
            }
        }
    }
}

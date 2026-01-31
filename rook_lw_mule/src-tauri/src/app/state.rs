use url::Url;


#[derive(Debug)]
pub struct AppState {
    home_url: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            home_url: None 
        }
    }

    pub fn get_home_url<'a>(&'a self) -> &'a Option<String> {
        &self.home_url
    }

    pub fn page_loaded(self: &mut Self, url: &Url) {
        println!("Yup its loaded: {}", url);
        if let None = self.home_url {
            self.home_url = Some(url.to_string());
        }
    }
}
use arguments;

#[derive(Debug)]
pub struct Account {
    pub name: String,
    pub username: String,
    pub password: String,
}

impl From<arguments::Account> for Account {
    fn from(a: arguments::Account) -> Self {
        Account {
            name: a.name,
            username: a.username,
            password: a.password,
        }
    }
}
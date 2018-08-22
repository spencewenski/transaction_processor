use arguments;

#[derive(Debug)]
pub struct Account {
    pub name: String,
    pub credentials: Option<AccountCredentials>,
}

#[derive(Debug)]
pub struct AccountCredentials {
    pub username: String,
    pub password: String,
}

impl From<arguments::Account> for Account {
    fn from(a: arguments::Account) -> Self {
        Account {
            name: a.name,
            credentials: if let Option::Some(ref c) = a.credentials {
                Option::Some(AccountCredentials {
                    username: c.username.to_owned(),
                    password: c.password.to_owned(),
                })
            } else {
                Option::None
            }
        }
    }
}
enum Account {
    FriendCode(String),
    ProfileName(String),
}

impl Account {
    fn get_link(&self) -> String {
        match self {
            Self::FriendCode(id) => {
                format!("https://steamcommunity.com/profiles/{}", id)
            }
            Self::ProfileName(profile_name) => {
                format!("https://steamcommunity.com/id/{}", profile_name)
            }
        }
    }

    //Scores the account based on games most played and location
    //Returns a float from 0-1
    fn score_based_on(&self, base: Account) -> f32 {
        return 1.0;
    }
}

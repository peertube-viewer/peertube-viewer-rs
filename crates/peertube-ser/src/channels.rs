use nanoserde::DeJson;

use super::common::Avatar;

#[derive(DeJson, Debug)]
#[allow(non_snake_case)]
pub struct Channel {
    pub id: i64,
    pub name: String,
    pub displayName: String,
    pub description: Option<String>,
    pub host: String,
    pub followingCount: i64,
    pub followersCount: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub ownerAccount: Account,
}

#[derive(DeJson, Debug)]
#[allow(non_snake_case)]
pub struct Account {
    pub id: u64,
    pub name: String,
    pub displayName: String,
    pub description: Option<String>,
    pub host: String,
    pub Avatar: Option<Avatar>,
    pub followingCount: u64,
    pub followersCount: u64,
    pub createdAt: String,
    pub updatedAt: String,
}

#[derive(DeJson, Debug)]
#[allow(non_snake_case)]
pub struct Channels {
    pub total: usize,
    pub data: Vec<Channel>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deser() {
        let test_data = r##"{"total":70,"data":[{"name":"1cd6752b-a43d-40c2-86ae-13ce538f25b9","host":"peertube.social","avatar":null,"id":1036,"hostRedundancyAllowed":false,"followingCount":0,"followersCount":0,"createdAt":"2018-11-15T17:58:28.154Z","updatedAt":"2018-11-15T17:58:28.154Z","displayName":"Test","description":"Test","support":null,"isLocal":false,"ownerAccount":{"name":"wiedaschaun","host":"peertube.social","avatar":{"path":"/lazy-static/avatars/f818f180-203d-4c98-aea9-1d809174a1fa.jpg","createdAt":"2018-11-15T17:58:28.133Z","updatedAt":"2018-11-15T17:58:28.133Z"},"id":2281,"hostRedundancyAllowed":false,"followingCount":0,"followersCount":0,"createdAt":"2018-11-15T17:58:28.142Z","updatedAt":"2018-11-15T17:58:28.142Z","displayName":"wiedaschaun","description":"wiedaschaun im Sinne von noch einmal :-)\nAufnahmen von Events und Vorträgen\nbei denen es um Freie Software geht.\n"}},{"name":"testing","host":"peertube.servebeer.com","avatar":null,"id":7626,"hostRedundancyAllowed":false,"followingCount":0,"followersCount":0,"createdAt":"2019-05-12T10:48:24.644Z","updatedAt":"2019-05-12T10:48:24.644Z","displayName":"test","description":null,"support":null,"isLocal":false,"ownerAccount":{"name":"nesquiik","host":"peertube.servebeer.com","avatar":{"path":"/lazy-static/avatars/5f7c1d98-10d6-48d4-9323-f0a643cc1550.jpg","createdAt":"2020-03-01T20:01:02.470Z","updatedAt":"2020-03-01T20:01:02.470Z"},"id":752,"hostRedundancyAllowed":false,"followingCount":8,"followersCount":5,"createdAt":"2018-11-15T14:28:13.269Z","updatedAt":"2019-06-01T07:56:29.763Z","displayName":"Nesquiik","description":"Config PC\nCPU : AMD Reyzen 5 2600X\nGPU : Nvidia GeForce RTX 2070\nEcran : Samsung TV 32\"\nSouris : Logitech G402\nClavier : Logitech G15\nGamepad : Xbox Controller \nMicro : Bird UM1\nCasque : Logitech G430"}}]}"##;
        // Exemple data from video.ploud.fr

        let _: Channels = DeJson::deserialize_json(test_data).unwrap();
    }
}

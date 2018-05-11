#[derive(Deserialize, Serialize, Debug)]
pub enum Message {
    Hello(String),
    GoodBye,
}

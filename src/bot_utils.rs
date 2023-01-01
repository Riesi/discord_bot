use std::error;
use serde_yaml;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub token: String,
}

pub fn write_example_credentials(){
    let cred = Credentials{
            token:"<fancy_token>".to_string(),
        };
        let f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open("bot_credentials.yml")
            .expect("Couldn't open file.");
        serde_yaml::to_writer(f, &cred).unwrap();
        println!("Failed to read credential file!\nExample file written instead.");
}

pub fn read_credentials() -> Result<Credentials, Box<dyn error::Error>>{
    let f = std::fs::File::open("./bot_credentials.yml")?;
    Ok::<Credentials, _>(serde_yaml::from_reader(f)?)
}
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Config {
    secret: String,
    dynamo_config: DynamoConfig,
}

#[derive(Deserialize, Serialize)]
struct ServerConfig {
    addr: String,
    port: u16,
}

#[derive(Deserialize, Serialize)]
struct DynamoConfig {

}

#[derive(Deserialize, Serialize)]
struct JWTConfig {

}

impl ::std::default::Default for MyConfig {
    fn default() -> Self { 
        Self { 
            version: 0, 
            api_key: "".into() 
        } 
    }
}
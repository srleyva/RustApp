extern crate user_service;

use user_service::user::user_service_client::UserServiceClient;
use user_service::user::{
    NewUserRequest, 
    AuthRequest, 
    
    User,
    Location,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
// creating a channel ie connection to server
    let channel = tonic::transport::Channel::from_static("http://127.0.0.1:8080")
    .connect()
    .await?;
// creating gRPC client from channel
    let mut client = UserServiceClient::new(channel);
// creating a new Request
    let new_request = tonic::Request::new(
        NewUserRequest {
            user: Some(User {
                first_name: "Stephen".to_string(),
                last_name: "Leyva".to_string(),
                username: "sleyva".to_string(),
                password: "password".to_string(),
                uid: "".to_string(),
                age: 22,
                gender: false,
                location: Some(Location {
                    latitude: 47.606209,
                    longitude: -122.332069,
                })
            })
        },
    );
    let response = client.new_user(new_request).await?.into_inner();
    println!("RESPONSE={:?}", response);

    let auth_request = tonic::Request::new(
        AuthRequest{
            username: "sleyva".to_string(),
            password: "password".to_string(),
        }
    );

    let response = client.auth(auth_request).await?.into_inner();
    println!("RESPONSE={:?}", response);

    let bad_auth_request = tonic::Request::new(
        AuthRequest{
            username: "sleyva".to_string(),
            password: "bad".to_string(),
        }
    );

    let response = client.auth(bad_auth_request).await?.into_inner();
    println!("RESPONSE={:?}", response);
    Ok(())
}
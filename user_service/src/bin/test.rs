/*
These test aren't great, but they cover what I want them too cover

Want to look into frameworks, but for now this is good enough
*/

extern crate user_service;

use user_service::user::user_service_client::UserServiceClient;
use user_service::user::{
    NewUserRequest, 
    AuthRequest, 
    
    User,
    Location,
    Gender,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = tonic::transport::Channel::from_static("http://user-service:8080")
        .connect()
        .await.unwrap();
    let mut client = UserServiceClient::new(channel);

    integration_tests::setup(&mut client).await;

    // Probably a way to do these async but mut client
    integration_tests::auth::test_auth_ok(&mut client).await;
    integration_tests::auth::test_auth_not_ok(&mut client).await;
    integration_tests::auth::test_auth_fails_no_username_or_no_password(&mut client).await;

    integration_tests::new_user::test_new_user_ok(&mut client).await;
    integration_tests::new_user::test_new_user_already_exists(&mut client).await;

    Ok(())
}

mod integration_tests {
    use super::*;

    pub async fn setup(client: &mut UserServiceClient<tonic::transport::Channel>) {
        let new_request = tonic::Request::new(
            NewUserRequest {
                user: Some(User {
                    first_name: "Test".to_string(),
                    last_name: "Test".to_string(),
                    username: "test".to_string(),
                    password: "test".to_string(),
                    uid: "".to_string(),
                    age: 22,
                    gender: Gender::Male as i32,
                    location: Some(Location {
                        latitude: 47.606209,
                        longitude: -122.332069,
                    })
                })
            },
        );
        client.new_user(new_request).await.unwrap();
    }

    pub mod new_user {
        use super::*;

        pub async fn test_new_user_ok(client: &mut UserServiceClient<tonic::transport::Channel>) {
            let new_request = tonic::Request::new(
                NewUserRequest {
                    user: Some(User {
                        first_name: "Stephen".to_string(),
                        last_name: "Leyva".to_string(),
                        username: "sleyva".to_string(),
                        password: "test".to_string(),
                        uid: "".to_string(),
                        age: 22,
                        gender: Gender::Male as i32,
                        location: Some(Location {
                            latitude: 47.606209,
                            longitude: -122.332069,
                        })
                    })
                },
            );
            client.new_user(new_request).await.unwrap();
            println!("test_new_user_ok: Ok");
        }

        pub async fn test_new_user_already_exists(client: &mut UserServiceClient<tonic::transport::Channel>) {
            let new_request = tonic::Request::new(
                NewUserRequest {
                    user: Some(User {
                        first_name: "Test".to_string(),
                        last_name: "Test".to_string(),
                        username: "test".to_string(),
                        password: "test".to_string(),
                        uid: "".to_string(),
                        age: 22,
                        gender: Gender::Male as i32,
                        location: Some(Location {
                            latitude: 47.606209,
                            longitude: -122.332069,
                        })
                    })
                },
            );

            client.new_user(new_request).await.unwrap_err();
            println!("test_new_user_already_exists: Ok");
        }
    }

    pub mod auth {
        use super::*;

        pub async fn test_auth_ok(client: &mut UserServiceClient<tonic::transport::Channel>) {
            let auth_request = tonic::Request::new(
                AuthRequest{
                    username: "test".to_string(),
                    password: "test".to_string(),
                }
            );
            client.auth(auth_request).await.unwrap();
    
            println!("test_auth_ok: Ok");
        }
    
        pub async fn test_auth_not_ok(client: &mut UserServiceClient<tonic::transport::Channel>) {
            let auth_request = tonic::Request::new(
                AuthRequest{
                    username: "test".to_string(),
                    password: "bad-password".to_string(),
                }
            );
    
            client.auth(auth_request).await.unwrap_err();
            println!("test_auth_not_ok: Ok");
        }
    
        pub async fn test_auth_fails_no_username_or_no_password(client: &mut UserServiceClient<tonic::transport::Channel>) {
            let auth_request = tonic::Request::new(
                AuthRequest{
                    username: "".to_string(),
                    password: "bad-password".to_string(),
                }
            );
    
            client.auth(auth_request).await.unwrap_err();
            println!("test_auth_fails_no_username: Ok");
    
            let auth_request = tonic::Request::new(
                AuthRequest{
                    username: "test".to_string(),
                    password: "".to_string(),
                }
            );
    
            client.auth(auth_request).await.unwrap_err();
            println!("test_auth_fails_no_username_password: Ok");
        }
    }
    
}

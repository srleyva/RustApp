extern crate user_service;

use base64::encode as b64_encode;
use bcrypt::hash;
use rusoto_core::{
    Region,
    RusotoError
};
use rusoto_dynamodb::{
    DynamoDb, DynamoDbClient, 

    CreateTableInput,
    AttributeDefinition,
    KeySchemaElement,
    ProvisionedThroughput,
    Tag,

    DescribeTableInput,
    DescribeTableError,

    PutItemInput,
    PutItemError,
    GetItemInput,
    GetItemError,
    AttributeValue,
};
use std::time::SystemTime;
use std::collections::HashMap;
use tonic::{transport::Server, Request, Response, Status};
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use serde::{Deserialize, Serialize};
use log::{info};
use env_logger;
use user_service::user::user_service_server::{UserService, UserServiceServer};
use user_service::user::{
    User,
    Location,
    NewUserResponse,
    NewUserRequest,
    AuthResponse,
    AuthRequest
};
use uuid::Uuid;

const DEFAULT_COST: u32 = 10;
const SECRET: &str = "my-secret";
const DEFAULT_ALGORITHM: Algorithm = Algorithm::HS256;
const VALID_TIME_SEC: u64 = 8 * 60 * 60; // 8 hours in seconds

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: String,
    uid: String,
}

pub struct MainUserService {
    client: DynamoDbClient,
}

impl MainUserService {
    fn new(client: DynamoDbClient) -> Self {
        Self {
            client
        }
    }

    async fn create_table(&self) {
        info!("Checking if table exists");
        match self.client.describe_table(DescribeTableInput{
            table_name: "date-app-user-service".to_string(),
        }).await {
            Ok(_) => {
                info!("Table exists");
            },
            Err(err) => {
                match err {
                    RusotoError::Service(err) => {
                        match err {
                            DescribeTableError::ResourceNotFound(_) => {
                                info!("Table does not exist: Creating");
                                // Create table
                                let create_table_input = CreateTableInput {
                                    attribute_definitions: vec![
                                        AttributeDefinition{
                                            attribute_name: "username".to_string(),
                                            attribute_type: "S".to_string(),
                                        },
                                    ],
                                    billing_mode: None,
                                    global_secondary_indexes: None,
                                    key_schema: vec![
                                        KeySchemaElement{
                                            attribute_name: "username".to_string(),
                                            key_type: "HASH".to_string(),
                                        }
                                    ],
                                    local_secondary_indexes: None,
                                    provisioned_throughput: Some(ProvisionedThroughput{
                                        read_capacity_units: 1,
                                        write_capacity_units: 1
                                    }),
                                    sse_specification: None,
                                    stream_specification: None,
                                    table_name: "date-app-user-service".to_string(),
                                    tags: Some(vec![
                                        Tag{
                                            key: "service".to_string(),
                                            value: "user-service".to_string(),
                                        }
                                    ]),
                                };
                
                                match self.client.create_table(create_table_input).await {
                                    Ok(_) => (),
                                    Err(err) => panic!("error creating table: {}", err)
                                };
                            },
                            _ => panic!("error creating table: {}", err)

                        }
                    }
                    _ => panic!("error creating table: {}", err)
                }
                
            }
        }
    }

    async fn create_item_from_user(&self, user: User) -> Result<(), RusotoError<PutItemError>> {
        let mut put_item = PutItemInput::default();
        put_item.table_name = "date-app-user-service".to_string();
        put_item.condition_expression = Some(String::from("attribute_not_exists(username)"));
        
        let mut username_attr = AttributeValue::default();
        username_attr.s = Some(user.username);
        put_item.item.insert("username".to_string(), username_attr);

        let mut password_attr = AttributeValue::default();
        password_attr.s = Some(user.password);
        put_item.item.insert("password".to_string(), password_attr);

        let mut uid_attr = AttributeValue::default();
        uid_attr.s = Some(user.uid);
        put_item.item.insert("uid".to_string(), uid_attr);
         
        return match self.client.put_item(put_item).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        };
    }

    fn create_jwt_token(uid: String) -> String {
        let key = EncodingKey::from_base64_secret(b64_encode(SECRET).as_str()).unwrap();
        let header = Header::new(DEFAULT_ALGORITHM);
        let exp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => {
                let current_epoch = n.as_secs();
                format!("{}", current_epoch + VALID_TIME_SEC)
            },
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };

        let claims = Claims{
            exp: exp,
            uid: uid,
        };
        encode(&header, &claims, &key).unwrap()
    }

    async fn get_user(&self, username: String) -> Result<(String, String, String), RusotoError<GetItemError>> {
        let mut get_item = GetItemInput::default();
        get_item.consistent_read = Some(true);
        get_item.table_name = "date-app-user-service".to_string();

        let mut username_attr = AttributeValue::default();
        username_attr.s = Some(username);
        get_item.key.insert("username".to_string(), username_attr);


        return match self.client.get_item(get_item).await {
            Ok(item) => {
                let mut user = item.item.unwrap();
                Ok((
                    user.remove("uid").unwrap().s.unwrap(), 
                    user.remove("username").unwrap().s.unwrap(), 
                    user.remove("password").unwrap().s.unwrap()))
            },
            Err(err) => Err(err),
        };        
    }
}

#[tonic::async_trait]
impl UserService for MainUserService {
    async fn new_user(&self, request: Request<NewUserRequest>) -> Result<Response<NewUserResponse>, Status> {
        let mut user = (request.into_inner() as NewUserRequest).user.unwrap();
        user.uid = Uuid::new_v4().to_string();
        user.password = hash(user.password, DEFAULT_COST).unwrap();
        if user.username.is_empty() || user.password.is_empty() {
            return Err(Status::unauthenticated("please provide username and password"));
        }
        
        return match self.create_item_from_user(user).await {
            Ok(_) => Ok(Response::new(NewUserResponse{})),
            Err(err) => {
                match err {
                    RusotoError::Service(err) => {
                       match err {
                           PutItemError::ConditionalCheckFailed(err) => {
                                info!("conditional err: {}", err);
                                Err(Status::already_exists("user already exists"))
                           }
                           _ => {
                            info!("Err creating user: {}", err);
                            return Err(Status::internal("internal server error"));
                           }
                       }
                    },
                    _ => {
                        info!("Err creating user: {}", err);
                        return Err(Status::internal("internal server error"));
                    }
                }
                
            }
        }
    }

    async fn auth(&self, request: Request<AuthRequest>) -> Result<Response<AuthResponse>, Status> {
        let auth = request.into_inner();
        info!("Auth: Username: {} Password: ****", auth.username);

        if auth.username.is_empty() || auth.password.is_empty() {
            return Err(Status::permission_denied("Please provide username or password"));
        }

        let user = match self.get_user(auth.username).await {
            Ok(user) => user,
            Err(err) => {
                info!("Err getting user: {}", err);
                return Err(Status::internal("internal server error"));
            }
        };

        let hashed_password = user.2;
        let provided_password = auth.password;

        if !bcrypt::verify(provided_password.as_str(), hashed_password.as_str()).unwrap() {
            return Err(Status::permission_denied("Bad Username or password"));
        }
        
        info!("Auth: Username: {} Successful", user.1);
        Ok(Response::new(AuthResponse{
            jwt: MainUserService::create_jwt_token(user.0),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("Starting User-Service");
    let addr = "0.0.0.0:8080".parse().unwrap();

    let region = Region::Custom{
        name: "test-region-1".to_owned(),
        endpoint: "http://dynamodb-local:8000".to_owned(),
    };

    let client = DynamoDbClient::new(region);
    let user_service = MainUserService::new(client);
    user_service.create_table().await;

    info!("Server listening on {}", addr);

    let service = UserServiceServer::new(user_service);
    Server::builder()
        .add_service(service)
        .serve(addr)
        .await?;
    Ok(())
}
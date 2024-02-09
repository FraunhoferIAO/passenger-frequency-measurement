use chrono::{DateTime, FixedOffset}; 
use dotenv::dotenv; 
use paho_mqtt::{Client as MqttClient, ConnectOptionsBuilder, SslOptionsBuilder}; 
use serde::{Deserialize, Serialize}; 
use std::{env, process, str, time::Duration}; 
use tokio_postgres::{Config, NoTls}; 
use uuid::Uuid; 

#[derive(Serialize, Deserialize, Debug)] 
struct TTNDeviceIds { 
    device_id: String, 
    application_ids: TTNApplicationIds, 
    dev_addr: String, 
} 

#[derive(Serialize, Deserialize, Debug)] 
struct TTNApplicationIds { 
    application_id: String, 
} 

#[derive(Serialize, Deserialize, Debug)] 
struct TTNLocation { 
    latitude: f64, 
    longitude: f64, 
    altitude: f64, 
    source: String, 
} 

#[derive(Serialize, Deserialize, Debug)] 
struct TTNGatewayId { 
    gateway_id: String, 
    eui: String, 
} 

#[derive(Serialize, Deserialize, Debug)] 
struct TTNMetaData { 
    gateway_ids: TTNGatewayId, 
    rssi: f64, 
    channel_rssi: f64, 
    snr: Option<f64>, 
    location: TTNLocation, 
} 

#[derive(Serialize, Deserialize, Debug)] 
struct TTNUplinkMessage { 
    f_port: Option<i64>, 
    f_cnt: Option<i64>, 
    frm_payload: Option<String>, 
    rx_metadata: Vec<TTNMetaData>, 
    received_at: String, 
    consumed_airtime: String, 
} 

#[derive(Serialize, Deserialize, Debug)] 
struct TTNMessage { 
    end_device_ids: TTNDeviceIds, 
    received_at: String, 
    uplink_message: TTNUplinkMessage, 
} 

#[derive(Debug, Clone)] 
struct Message { 
    id: String, 
    device_id: String, 
    dev_addr: String, 
    payload: String, 
    received_at: DateTime<FixedOffset>, 
} 

#[tokio::main] 
async fn main() { 
    dotenv().ok(); 

    // POSTGRES CONFIG 
    let mut postgres_config = Config::new(); 
    postgres_config.dbname(&env::var("DB_NAME").expect("DB_NAME variable not set")); 
    postgres_config.host(&env::var("DB_HOST").expect("DB_HOST variable not set")); 
    postgres_config.port( 
        env::var("DB_PORT") 
            .expect("DB_PORT variable not set") 
            .parse() 
            .expect("DB_PORT parsing failed"), 
    ); 
    postgres_config.user(&env::var("DB_USER").expect("DB_USER variable not set")); 
    
    // POSTGRES CLIENT & CONNECTION 
    let (posgtres_client, connection) = postgres_config.connect(NoTls).await.unwrap(); 
    tokio::spawn(async move { 
        if let Err(e) = connection.await { 
            eprintln!("connection error: {}", e); 
        } 
    }); 

    // MQTT CLIENT 
    let cli = MqttClient::new(env::var("TTN_SERVER").expect("TTN_SERVER not set")).unwrap_or_else( 
        |err| { 
            println!("Error creating the client: {:?}", err); 
            process::exit(1); 
        }, 
    ); 

    // MQTT SSL OPTIONS 
    let ssl_opts = SslOptionsBuilder::new()
        .finalize(); 
 
    // MQTT CONNECTION OPTIONS 
    let conn_opts = ConnectOptionsBuilder::new() 
        .keep_alive_interval(Duration::from_secs(20)) 
        .clean_session(true) 
        .user_name(env::var("TTN_USER").expect("TTN_USER not set").as_str()) 
        .password( 
            env::var("TTN_PASSWORD") 
                .expect("TTN_PASSWORD not set") 
                .as_str(), 
        ) 
        .connect_timeout(Duration::from_secs(10)) 
        .ssl_options(ssl_opts) 
        .finalize(); 

    if let Err(e) = cli.connect(conn_opts) { 
        println!("Unable to connect:\n\t{:?}", e); 
        process::exit(1); 
    } 

    let rx = cli.start_consuming(); 

    // MQTT SUBSCRIPTION TO TOPIC 
    cli.subscribe("v3/+/devices/+/up", 0).unwrap(); 

    // CHECK EACH MESSAGE 
    for msg in rx.iter().flatten() { 
        let data = str::from_utf8(msg.payload()).unwrap(); 
        let deserialized: TTNMessage = serde_json::from_str(data).unwrap(); 
        println!("PAYLOAD: {:?}", deserialized); 

        // Only save message, if f_port == 1 
        if deserialized.uplink_message.f_port == Some(1) { 
            let event_time = 
                DateTime::parse_from_rfc3339(deserialized.uplink_message.received_at.as_str()) 
                    .unwrap(); 

            let my_uuid = Uuid::now_v7(); 

            let msg = Message { 
                id: my_uuid.hyphenated().to_string(), 
                device_id: deserialized.end_device_ids.device_id, 
                dev_addr: deserialized.end_device_ids.dev_addr, 
                payload: deserialized.uplink_message.frm_payload.unwrap(), 
                received_at: event_time, 
            }; 

            let _rows = posgtres_client 
                .query( 
                    "INSERT  
                    INTO ttn_payloads (id, device_id, dev_addr, payload, received_at)  
                    VALUES ($1, $2, $3, $4, $5)", 
                    &[ 
                        &msg.id, 
                        &msg.device_id, 
                        &msg.dev_addr, 
                        &msg.payload, 
                        &msg.received_at, 
                    ], 
                ) 
                .await 
                .unwrap(); 
        }; 
    } 
} 
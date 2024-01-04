// use serde_json::{json, Value};
use std::io::{self, };
use aws_config::{
    meta::region::RegionProviderChain,
    BehaviorVersion
};
// use aws_credential_types::provider::{ProvideCredentials, SharedCredentialsProvider};
use aws_sdk_ec2::types::{Instance, Reservation};

#[derive(Debug)]
struct InstanceInfo {
    instance_id: String,
    instance_type: String,
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let region_provider = RegionProviderChain::default_provider().or_else("ap-northeast-1");

    // memo: config の初期化の後に呼ぶと、region_provider が move しているのでコンパイルエラー
    let region = region_provider.region().await.unwrap();
    println!("Region: {:?}", region);

    // memo: こっちだとコンパイルは通るが API 呼び出し時にエラーとなる。理由はわからなかった
    // let config = aws_config::SdkConfig::builder()
    //     .behavior_version(BehaviorVersion::latest())
    //     .region(region)
    //     .retry_config(RetryConfig::standard().with_max_attempts(3))
    //     .build();

    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;


    let client = aws_sdk_ec2::Client::new(&config);
    list_instances(&client).await;

    Ok(())
}


async fn list_instances(client: &aws_sdk_ec2::Client) {
    let resp = client.describe_instances()
        .send()
        .await
        .unwrap_or_else(|err| {
            panic!("Error listing instances: {}", err.to_string());
        });

    let reservations = resp.reservations
        .unwrap_or(Vec::<Reservation>::new());

    // Vec<Reservation> から Vec<Instance> に変換したい
    // Reservation は .instances で Option<Vec<Instance>> を持っている
    let instances = reservations.into_iter()
        .map(|reservation| reservation.instances.unwrap_or(Vec::<Instance>::new()))
        .flatten()
        .collect::<Vec<Instance>>();

    let result = instances.into_iter()
        .map(|instance| InstanceInfo {
            instance_id: instance.instance_id.unwrap_or(String::from("")),
            instance_type: match instance.instance_type {
                Some(it) => String::from(it.as_str()),
                None => String::from(""),
            },
        })
        .collect::<Vec<InstanceInfo>>();

    // println!("{:#?}", result);
    result.iter().for_each(|i| {
        println!("instance_id: {}, instance_type: {}", i.instance_id, i.instance_type);
    });
}
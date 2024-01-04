// use serde_json::{json, Value};
use std::io::{self, };
use aws_config::{
    meta::region::RegionProviderChain,
    BehaviorVersion
};
// use aws_credential_types::provider::{ProvideCredentials, SharedCredentialsProvider};


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
        .unwrap_or_else(|| {
            panic!("Error listing instances");
        });

    let instances = reservations.iter().map(|reservation| {
        // reservation.instances.as_ref().unwrap().iter().for_each(|instance| {
        //     println!("Instance ID: {:?}", instance.instance_id.as_ref().unwrap());
        // });
        let is: Vec<InstanceInfo> = reservation.instances
            .as_ref()
            .unwrap_or_else(|| {
                panic!("Error listing instances");
            })
            .iter().map(|instance| {
                let id = instance.instance_id.as_ref().unwrap();
                let it = instance.instance_type.as_ref().unwrap();
                let instance_type = it.as_str().to_string();

                InstanceInfo {
                    instance_id: id.to_string(),
                    instance_type: instance_type.to_string(),
                }
            })
            .collect();
        is
    });

    instances.into_iter().for_each(|instance| {
        instance.iter().for_each(|i| {
            println!("Instance: Id={:?} Type={:?}", i.instance_id, i.instance_type);
        });
    });

    // println!("{:#?}", resp);
}
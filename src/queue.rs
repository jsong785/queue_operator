use k8s_openapi::api::apps::v1::{ReplicaSet, ReplicaSetSpec};
use k8s_openapi::api::core::v1::{Container, ContainerPort, PodSpec, PodTemplateSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector;
use kube::api::{DeleteParams, ObjectMeta, PostParams};
use kube::{Api, Client, Error};
use std::collections::BTreeMap;

pub async fn deploy(
    client: Client,
    name: &str,
    replicas: i32,
    namespace: &str,
) -> Result<ReplicaSet, Error> {
    let mut labels = BTreeMap::new();
    labels.insert("app".into(), name.into());

    let replicaset = ReplicaSet {
        metadata: ObjectMeta {
            name: Some(name.into()),
            namespace: Some(namespace.into()),
            labels: Some(labels.clone()),
            ..ObjectMeta::default()
        },
        spec: Some(ReplicaSetSpec {
            replicas: Some(replicas),
            selector: LabelSelector {
                match_expressions: None,
                match_labels: Some(labels.clone()),
            },
            template: Some(PodTemplateSpec {
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: name.into(),
                        image: Some("queue".into()),
                        ports: Some(vec![ContainerPort {
                            container_port: 8080,
                            ..ContainerPort::default()
                        }]),
                        image_pull_policy: Some("IfNotPresent".into()),
                        ..Container::default()
                    }],
                    ..PodSpec::default()
                }),
                metadata: Some(ObjectMeta {
                    labels: Some(labels),
                    ..ObjectMeta::default()
                }),
            }),
            ..ReplicaSetSpec::default()
        }),
        ..ReplicaSet::default()
    };

    // Create the deployment defined above
    let api = Api::namespaced(client, namespace);
    api.create(&PostParams::default(), &replicaset).await
}

pub async fn delete(client: Client, name: &str, namespace: &str) -> Result<(), Error> {
    let api: Api<ReplicaSet> = Api::namespaced(client, namespace);
    api.delete(name, &DeleteParams::default()).await?;
    Ok(())
}

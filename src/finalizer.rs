use crate::crd::Queue;
use kube::api::{Patch, PatchParams};
use kube::{Api, Client, Error};
use serde_json::{json, Value};

pub async fn add(client: Client, name: &str, namespace: &str) -> Result<Queue, Error> {
    let api = Api::namespaced(client, namespace);
    let finalizer = json!({
        "metadata": {
            "finalizers": ["queues.example.com/finalizer"]
        }
    });

    let patch: Patch<&Value> = Patch::Merge(&finalizer);
    api.patch(name, &PatchParams::default(), &patch).await
}

pub async fn delete(client: Client, name: &str, namespace: &str) -> Result<Queue, Error> {
    let api = Api::namespaced(client, namespace);
    let finalizer = json!({
        "metadata": {
            "finalizers": null
        }
    });

    let patch = Patch::Merge(&finalizer);
    api.patch(name, &PatchParams::default(), &patch).await
}

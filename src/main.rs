mod crd;
mod finalizer;
mod queue;

use crate::crd::Queue;
use std::sync::Arc;

use futures::stream::StreamExt;
use kube::runtime::watcher::Config;
use kube::Resource;
use kube::ResourceExt;
use kube::{client::Client, runtime::controller::Action, runtime::Controller, Api};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    let kubernetes_client: Client = Client::try_default()
        .await
        .expect("Expected a valid KUBECONFIG environment variable.");

    let crd_api = Api::all(kubernetes_client.clone());
    let client = Arc::new(kubernetes_client.clone());
    Controller::new(crd_api, Config::default())
        .run(reconcile, on_error, client)
        .for_each(|reconciliation_result| async move {
            match reconciliation_result {
                Ok(queue_resource) => {
                    println!("Reconciliation successful. Resource: {:?}", queue_resource);
                }
                Err(reconciliation_err) => {
                    eprintln!("Reconciliation error: {:?}", reconciliation_err)
                }
            }
        })
        .await;
}

enum QueueAction {
    Create,
    Delete,
    NoOp,
}

async fn reconcile(queue: Arc<Queue>, client: Arc<Client>) -> Result<Action, Error> {
    let namespace: String = match queue.namespace() {
        None => {
            return Err(Error::UserInputError(
                "Expected Queue resource to be namespaced. Can't deploy to an unknown namespace."
                    .to_owned(),
            ));
        }
        Some(namespace) => namespace,
    };
    let name = queue.name_any();

    let client = client.as_ref();
    match determine_action(&queue) {
        QueueAction::Create => {
            finalizer::add(client.clone(), &name, &namespace).await?;
            queue::deploy(client.clone(), &name, queue.spec.replicas, &namespace).await?;
            Ok(Action::requeue(Duration::from_secs(10)))
        }
        QueueAction::Delete => {
            queue::delete(client.clone(), &name, &namespace).await?;
            finalizer::delete(client.clone(), &name, &namespace).await?;
            Ok(Action::await_change())
        }
        QueueAction::NoOp => Ok(Action::requeue(Duration::from_secs(10))),
    }
}

fn determine_action(queue: &Queue) -> QueueAction {
    if queue.meta().deletion_timestamp.is_some() {
        QueueAction::Delete
    } else if queue
        .meta()
        .finalizers
        .as_ref()
        .map_or(true, |finalizers| finalizers.is_empty())
    {
        QueueAction::Create
    } else {
        QueueAction::NoOp
    }
}

fn on_error(queue: Arc<Queue>, error: &Error, _cclient: Arc<Client>) -> Action {
    eprintln!("Reconciliation error udder:\n{:?}.\n{:?}", error, queue);
    Action::requeue(Duration::from_secs(5))
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Kubernetes reported error: {source}")]
    KubeError {
        #[from]
        source: kube::Error,
    },
    #[error("Invalid Queue CRD: {0}")]
    UserInputError(String),
}

use azure_core::Context;
use azure_cosmos::prelude::*;
use futures::stream::StreamExt;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // First we retrieve the account name and master key from environment variables.
    // We expect master keys (ie, not resource constrained)
    let master_key =
        std::env::var("COSMOS_MASTER_KEY").expect("Set env variable COSMOS_MASTER_KEY first!");
    let account = std::env::var("COSMOS_ACCOUNT").expect("Set env variable COSMOS_ACCOUNT first!");

    let database_name = std::env::args()
        .nth(1)
        .expect("please specify database name as first command line parameter");

    // This is how you construct an authorization token.
    // Remember to pick the correct token type.
    // Here we assume master.
    // Most methods return a ```Result<_, azure_cosmos::Error>```.
    // ```azure_cosmos::Error``` is an enum union of all the possible underlying
    // errors, plus Azure specific ones. For example if a REST call returns the
    // unexpected result (ie NotFound instead of Ok) we return an Err telling
    // you that.
    let authorization_token = permission::AuthorizationToken::primary_from_base64(&master_key)?;

    // Once we have an authorization token you can create a client instance. You can change the
    // authorization token at later time if you need, for example, to escalate the privileges for a
    // single operation.
    let client = CosmosClient::new(account, authorization_token, CosmosOptions::default());

    // The Cosmos' client exposes a lot of methods. This one lists the databases in the specified
    // account. Database do not implement Display but deref to &str so you can pass it to methods
    // both as struct or id.

    let mut list_databases_stream = Box::pin(client.list_databases().into_stream());
    while let Some(list_databases_response) = list_databases_stream.next().await {
        println!("list_databases_response = {:#?}", list_databases_response?);
    }
    drop(list_databases_stream);

    let db = client.create_database(&database_name).into_future().await?;
    println!("created database = {:#?}", db);

    // create collection!
    {
        let db_client = client.clone().into_database_client(database_name.clone());

        let create_collection_response = db_client
            .create_collection(
                Context::new(),
                "panzadoro",
                CreateCollectionOptions::new("/id"),
            )
            .await?;

        println!(
            "create_collection_response == {:#?}",
            create_collection_response
        );

        let db_collection = db_client.clone().into_collection_client("panzadoro");

        let get_collection_response = db_collection
            .get_collection(Context::new(), GetCollectionOptions::new())
            .await?;
        println!("get_collection_response == {:#?}", get_collection_response);

        let stream = db_client.list_collections(Context::new(), ListCollectionsOptions::new());
        let mut stream = Box::pin(stream);
        while let Some(res) = stream.next().await {
            let res = res?;
            println!("res == {:#?}", res);
        }

        let delete_response = db_collection
            .delete_collection(Context::new(), DeleteCollectionOptions::new())
            .await?;
        println!("collection deleted: {:#?}", delete_response);
    }

    let resp = client
        .into_database_client(database_name)
        .delete_database(Context::new(), DeleteDatabaseOptions::new())
        .await?;
    println!("database deleted. resp == {:#?}", resp);

    Ok(())
}
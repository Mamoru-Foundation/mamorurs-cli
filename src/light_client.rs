// use crate::proto::validation_chain::{
//     query_client::QueryClient as GeneratedQueryClient, QueryAllChainRequest, QueryAllChainResponse,
// };
use mamoru_chain_client::PageRequest;
use tonic::transport::Channel;

// use crate::proto::cosmos::tx::v1beta1::service_client::ServiceClient as TxClient;

// trait DeserializableMessage: MessageExt + Default + Sized + TypeUrl {}

// impl<T: MessageExt + Default + Sized + TypeUrl> DeserializableMessage for T {}

pub struct QueryClientLight {
    client: GeneratedQueryClient<tonic::transport::Channel>,
    #[allow(dead_code)]
    tx_client: TxClient<tonic::transport::Channel>,
}

impl QueryClientLight {
    pub async fn connect(grpc_url: String) -> Result<Self, tonic::transport::Error> {
        let channel = Channel::builder(grpc_url.parse().unwrap())
            .connect()
            .await
            .unwrap();
        // let client: GeneratedQueryClient<tonic::transport::Channel> =
        //     GeneratedQueryClient::connect(grpc_url.to_string()).await?;
        let limit = 10 * 1024 * 1024;
        let client = GeneratedQueryClient::new(channel)
            .max_encoding_message_size(limit)
            .max_decoding_message_size(limit);

        let auth_channel = Channel::builder(grpc_url.parse().unwrap())
            .connect()
            .await
            .unwrap();
        let tx_client = TxClient::new(auth_channel);

        Ok(Self { client, tx_client })
    }

    pub async fn list_chains(&self) -> Result<QueryAllChainResponse, tonic::Status> {
        let request = tonic::Request::new(QueryAllChainRequest {
            pagination: Some(PageRequest {
                count_total: true,
                limit: 100,
                offset: 0,
                ..Default::default()
            }),
        });
        let mut client = self.client.clone();
        let response = client.chain_all(request).await;
        match response {
            Ok(response) => Ok(response.into_inner()),
            Err(err) => Err(err),
        }
    }

    // pub async fn get_tx<R>(&self, tx_hash: String) -> Result<Vec<R>, tonic::Status>
    // {
    //     let mut client = self.tx_client.clone();

    //     let tx_response = client
    //         .get_tx(GetTxRequest {
    //             hash: tx_hash.clone(),
    //         })
    //         .await;

    //     match tx_response {
    //         Ok(response) => {
    //             let get_tx_response = response.into_inner();
    //             let tx_response = get_tx_response.tx_response.expect("Always exists.");
    //             let tx_response_objects = make_responses(&tx_response.data);

    //             Ok(tx_response_objects)
    //         }
    //         Err(err) => Err(err),
    //     }
    // }
}

// fn make_responses<R: DeserializableMessage>(data: &str) -> Vec<R> {
//     let bytes =
//         hex::decode(data).expect("BUG: Cosmos SDK returned invalid non-hex TxResponse::data.");
//     let res = TxMsgData::decode(bytes.as_slice())
//         .expect("BUG: Cosmos SDK returned non `TxMsgData` in TxResponse::data.");

//     res.msg_responses
//         .into_iter()
//         .map(|resp| R::from_any(&resp).expect("BUG: incompatible type conversion."))
//         .collect()
// }

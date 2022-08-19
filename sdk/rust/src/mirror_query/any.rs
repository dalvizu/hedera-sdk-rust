/*
 * ‌
 * Hedera Rust SDK
 * ​
 * Copyright (C) 2022 - 2023 Hedera Hashgraph, LLC
 * ​
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * ‍
 */

use async_trait::async_trait;
use hedera_proto::{
    mirror,
    services,
};
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};
use tonic::transport::Channel;
use tonic::{
    Code,
    Status,
    Streaming,
};

use crate::mirror_query::MirrorQuerySubscribe;
use crate::topic::TopicMessageQueryData;
use crate::{
    FromProtobuf,
    MirrorQuery,
    NodeAddress,
    NodeAddressBookQueryData,
    TopicMessage,
};

pub type AnyMirrorQuery = MirrorQuery<AnyMirrorQueryData>;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase", tag = "$type")]
pub enum AnyMirrorQueryData {
    NodeAddressBook(NodeAddressBookQueryData),
    TopicMessage(TopicMessageQueryData),
}

pub type AnyMirrorQueryResponse = Vec<AnyMirrorQueryMessage>;

#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase", tag = "$type")]
pub enum AnyMirrorQueryMessage {
    NodeAddressBook(NodeAddress),
    TopicMessage(TopicMessage),
}

pub enum AnyMirrorQueryGrpcMessage {
    NodeAddressBook(services::NodeAddress),
    TopicMessage(mirror::ConsensusTopicResponse),
}

pub enum AnyMirrorQueryGrpcStream {
    NodeAddressBook(Streaming<services::NodeAddress>),
    TopicMessage(Streaming<mirror::ConsensusTopicResponse>),
}

#[async_trait]
impl MirrorQuerySubscribe for AnyMirrorQueryData {
    type GrpcStream = AnyMirrorQueryGrpcStream;

    type GrpcMessage = AnyMirrorQueryGrpcMessage;

    type Message = AnyMirrorQueryMessage;

    fn should_retry(&self, status_code: Code) -> bool {
        match self {
            Self::NodeAddressBook(query) => query.should_retry(status_code),
            Self::TopicMessage(query) => query.should_retry(status_code),
        }
    }

    async fn subscribe(&self, channel: Channel) -> Result<Self::GrpcStream, Status> {
        match self {
            Self::NodeAddressBook(query) => {
                query.subscribe(channel).await.map(AnyMirrorQueryGrpcStream::NodeAddressBook)
            }

            Self::TopicMessage(query) => {
                query.subscribe(channel).await.map(AnyMirrorQueryGrpcStream::TopicMessage)
            }
        }
    }

    async fn message(
        &self,
        stream: &mut Self::GrpcStream,
    ) -> Result<Option<Self::GrpcMessage>, Status> {
        match stream {
            AnyMirrorQueryGrpcStream::NodeAddressBook(stream) => stream
                .message()
                .await
                .map(|message| message.map(AnyMirrorQueryGrpcMessage::NodeAddressBook)),

            AnyMirrorQueryGrpcStream::TopicMessage(stream) => stream
                .message()
                .await
                .map(|message| message.map(AnyMirrorQueryGrpcMessage::TopicMessage)),
        }
    }
}

impl FromProtobuf<AnyMirrorQueryGrpcMessage> for AnyMirrorQueryMessage {
    fn from_protobuf(message: AnyMirrorQueryGrpcMessage) -> crate::Result<Self>
    where
        Self: Sized,
    {
        match message {
            AnyMirrorQueryGrpcMessage::NodeAddressBook(message) => {
                NodeAddress::from_protobuf(message).map(Self::NodeAddressBook)
            }

            AnyMirrorQueryGrpcMessage::TopicMessage(message) => {
                TopicMessage::from_protobuf(message).map(Self::TopicMessage)
            }
        }
    }
}

// NOTE: as we cannot derive serde on MirrorQuery<T> directly as `T`,
//  we create a proxy type that has the same layout but is only for AnyMirrorQueryData and does
//  derive(Deserialize).

#[derive(serde::Deserialize, serde::Serialize)]
struct AnyMirrorQueryProxy {
    #[serde(flatten)]
    data: AnyMirrorQueryData,
}

impl<D> Serialize for MirrorQuery<D>
where
    D: MirrorQuerySubscribe,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO: remove the clones, should be possible with Cows

        AnyMirrorQueryProxy { data: self.data.clone().into() }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AnyMirrorQuery {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <AnyMirrorQueryProxy as Deserialize>::deserialize(deserializer)
            .map(|query| Self { data: query.data })
    }
}

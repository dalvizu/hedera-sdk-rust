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

use std::fmt::{
    self,
    Debug,
    Display,
    Formatter,
};
use std::str::FromStr;

use hedera_proto::services;
use rand::{
    thread_rng,
    Rng,
};
use time::{
    Duration,
    OffsetDateTime,
};

use crate::{
    AccountId,
    Error,
    FromProtobuf,
    ToProtobuf,
};

/// The client-generated ID for a transaction.
///
/// This is used for retrieving receipts and records for a transaction, for appending to a file
/// right after creating it, for instantiating a smart contract with bytecode in a file just created,
/// and internally by the network for detecting when duplicate transactions are submitted.
///
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "ffi", derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr))]
pub struct TransactionId {
    /// The account that pays for this transaction.
    pub account_id: AccountId,

    /// The time from when this transaction is valid.
    ///
    /// When a transaction is submitted there is additionally a
    /// [`valid_duration`](crate::Transaction::transaction_valid_duration) (defaults to 120s)
    /// and together they define a time window that a transaction may be processed in.
    pub valid_start: OffsetDateTime,

    /// Nonce for this transaction.
    pub nonce: Option<i32>,

    /// `true` if the transaction is `scheduled`.
    pub scheduled: bool,
}

impl TransactionId {
    /// Generates a new transaction ID for the given account ID.
    #[must_use]
    pub fn generate(account_id: AccountId) -> Self {
        let valid_start = OffsetDateTime::now_utc()
            - Duration::nanoseconds(thread_rng().gen_range(5_000_000_000..8_000_000_000));

        Self { account_id, valid_start, scheduled: false, nonce: None }
    }
}

impl Debug for TransactionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self)
    }
}

impl Display for TransactionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}@{}.{}{}{}",
            self.account_id,
            self.valid_start.unix_timestamp(),
            self.valid_start.nanosecond(),
            if self.scheduled { "?scheduled" } else { "" },
            self.nonce.map(|nonce| format!("/{}", nonce)).as_deref().unwrap_or_default()
        )
    }
}

impl FromProtobuf<services::TransactionId> for TransactionId {
    fn from_protobuf(pb: services::TransactionId) -> crate::Result<Self> {
        let account_id = pb_getf!(pb, account_id)?;
        let account_id = AccountId::from_protobuf(account_id)?;

        let valid_start = pb_getf!(pb, transaction_valid_start)?;

        Ok(Self {
            account_id,
            valid_start: valid_start.into(),
            nonce: (pb.nonce != 0).then_some(pb.nonce),
            scheduled: pb.scheduled,
        })
    }
}

// TODO: add unit tests to prove parsing
// TODO: potentially improve parsing with `nom` or `combine`
impl FromStr for TransactionId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const EXPECTED: &str = "expecting <accountId>@<validStart>[?scheduled][/<nonce>]";

        let mut parts = s.splitn(4, &['@', '?', '/']);

        let account_id = if let Some(account_id_s) = parts.next() {
            AccountId::from_str(account_id_s)?
        } else {
            return Err(Error::basic_parse(EXPECTED));
        };

        let valid_start = if let Some(valid_start_s) = parts.next() {
            let (seconds_s, nanos_s) =
                valid_start_s.split_once('.').ok_or_else(|| Error::basic_parse(EXPECTED))?;

            let seconds = i64::from_str(seconds_s).map_err(Error::basic_parse)?;
            let nanos = i64::from_str(nanos_s).map_err(Error::basic_parse)?;

            OffsetDateTime::from_unix_timestamp(seconds).map_err(Error::basic_parse)?
                + Duration::nanoseconds(nanos)
        } else {
            return Err(Error::basic_parse(EXPECTED));
        };

        let mut scheduled = false;
        let mut nonce = None;

        for part in parts.take(2) {
            if part == "scheduled" {
                scheduled = true;
            } else {
                nonce = Some(i32::from_str(part).map_err(Error::basic_parse)?);
            }
        }

        Ok(Self { account_id, valid_start, nonce, scheduled })
    }
}

impl ToProtobuf for TransactionId {
    type Protobuf = services::TransactionId;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::TransactionId {
            account_id: Some(self.account_id.to_protobuf()),
            scheduled: self.scheduled,
            nonce: self.nonce.unwrap_or_default(),
            transaction_valid_start: Some(self.valid_start.into()),
        }
    }
}

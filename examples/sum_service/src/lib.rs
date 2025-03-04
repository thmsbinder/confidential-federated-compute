// Copyright 2023 Google LLC.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![no_std]

extern crate alloc;

use alloc::{boxed::Box, format, vec};
use byteorder::{ByteOrder, LittleEndian};
use oak_crypto::signer::Signer;
use pipeline_transforms::{
    io::{EncryptionMode, RecordDecoder, RecordEncoder},
    proto::{
        ConfigureAndAttestRequest, ConfigureAndAttestResponse, GenerateNoncesRequest,
        GenerateNoncesResponse, PipelineTransform, TransformRequest, TransformResponse,
    },
};

pub struct SumService {
    signer: Box<dyn Signer>,
    record_decoder: Option<RecordDecoder>,
    record_encoder: RecordEncoder,
}

impl SumService {
    pub fn new(signer: Box<dyn Signer>) -> Self {
        Self { signer, record_decoder: None, record_encoder: RecordEncoder }
    }
}

impl PipelineTransform for SumService {
    fn configure_and_attest(
        &mut self,
        _request: ConfigureAndAttestRequest,
    ) -> Result<ConfigureAndAttestResponse, micro_rpc::Status> {
        self.record_decoder =
            Some(RecordDecoder::create(|msg| Ok(self.signer.sign(msg))).map_err(|err| {
                micro_rpc::Status::new_with_message(
                    micro_rpc::StatusCode::Internal,
                    format!("failed to create RecordDecoder: {:?}", err),
                )
            })?);
        Ok(ConfigureAndAttestResponse {
            public_key: self.record_decoder.as_ref().unwrap().public_key().to_vec(),
            ..Default::default()
        })
    }

    fn generate_nonces(
        &mut self,
        request: GenerateNoncesRequest,
    ) -> Result<GenerateNoncesResponse, micro_rpc::Status> {
        let record_decoder = self.record_decoder.as_mut().ok_or_else(|| {
            micro_rpc::Status::new_with_message(
                micro_rpc::StatusCode::FailedPrecondition,
                "service has not been configured",
            )
        })?;
        let count: usize = request.nonces_count.try_into().map_err(|err| {
            micro_rpc::Status::new_with_message(
                micro_rpc::StatusCode::InvalidArgument,
                format!("nonces_count is invalid: {:?}", err),
            )
        })?;
        Ok(GenerateNoncesResponse { nonces: record_decoder.generate_nonces(count) })
    }

    fn transform(
        &mut self,
        request: TransformRequest,
    ) -> Result<TransformResponse, micro_rpc::Status> {
        let record_decoder = self.record_decoder.as_mut().ok_or_else(|| {
            micro_rpc::Status::new_with_message(
                micro_rpc::StatusCode::FailedPrecondition,
                "service has not been configured",
            )
        })?;

        let mut sum: u64 = 0;
        for input in &request.inputs {
            let data = record_decoder.decode(input).map_err(|err| {
                micro_rpc::Status::new_with_message(
                    micro_rpc::StatusCode::InvalidArgument,
                    format!("failed to decode input: {:?}", err),
                )
            })?;

            if data.len() != 8 {
                return Err(micro_rpc::Status::new_with_message(
                    micro_rpc::StatusCode::InvalidArgument,
                    "input must be 8 bytes",
                ));
            }
            sum = sum.checked_add(LittleEndian::read_u64(&data)).ok_or_else(|| {
                micro_rpc::Status::new_with_message(
                    micro_rpc::StatusCode::InvalidArgument,
                    "addition overflow",
                )
            })?;
        }

        let mut buffer = [0; 8];
        LittleEndian::write_u64(&mut buffer, sum);

        // SumService always produces unencrypted outputs.
        let output =
            self.record_encoder.encode(EncryptionMode::Unencrypted, &buffer).map_err(|err| {
                micro_rpc::Status::new_with_message(
                    micro_rpc::StatusCode::Internal,
                    format!("failed to encode output: {:?}", err),
                )
            })?;
        Ok(TransformResponse { outputs: vec![output], num_ignored_inputs: 0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use coset::{CborSerializable, CoseSign1};
    use oak_restricted_kernel_sdk::testing::MockSigner;
    use pipeline_transforms::proto::Record;
    use sha2::{Digest, Sha256};

    /// Helper function to create a SumService.
    fn create_sum_service() -> SumService {
        SumService::new(Box::new(MockSigner::create().unwrap()))
    }

    /// Helper function to convert data to an unencrypted Record.
    fn encode_unencrypted(data: &[u8]) -> Record {
        RecordEncoder::default().encode(EncryptionMode::Unencrypted, data).unwrap()
    }

    #[test]
    fn test_configure_and_attest() -> Result<(), micro_rpc::Status> {
        struct FakeSigner;
        impl Signer for FakeSigner {
            fn sign(&self, message: &[u8]) -> Vec<u8> {
                Sha256::digest(message).to_vec()
            }
        }

        let mut service = SumService::new(Box::new(FakeSigner));
        let response = service.configure_and_attest(ConfigureAndAttestRequest::default())?;
        assert!(!response.public_key.is_empty());
        CoseSign1::from_slice(&response.public_key)
            .unwrap()
            .verify_signature(b"", |signature, message| {
                anyhow::ensure!(signature == Sha256::digest(message).as_slice());
                Ok(())
            })
            .expect("signature mismatch");
        Ok(())
    }

    #[test]
    fn test_generate_nonces() -> Result<(), micro_rpc::Status> {
        let mut service = create_sum_service();
        service.configure_and_attest(ConfigureAndAttestRequest::default())?;
        let response = service.generate_nonces(GenerateNoncesRequest { nonces_count: 3 })?;
        assert_eq!(response.nonces.len(), 3);
        assert_ne!(response.nonces[0], b"");
        assert_ne!(response.nonces[1], b"");
        assert_ne!(response.nonces[2], b"");
        Ok(())
    }

    #[test]
    fn test_generate_nonces_without_configure() {
        let mut service = create_sum_service();
        assert!(service.generate_nonces(GenerateNoncesRequest { nonces_count: 3 }).is_err());
    }

    #[test]
    fn test_generate_nonces_with_invalid_count() -> Result<(), micro_rpc::Status> {
        let mut service = create_sum_service();
        service.configure_and_attest(ConfigureAndAttestRequest::default())?;
        assert!(service.generate_nonces(GenerateNoncesRequest { nonces_count: -1 }).is_err());
        Ok(())
    }

    #[test]
    fn test_transform_without_configure() {
        let mut service = create_sum_service();
        let request = TransformRequest {
            inputs: vec![encode_unencrypted(&[1, 0, 0, 0, 0, 0, 0, 0]); 2],
            ..Default::default()
        };
        assert!(service.transform(request).is_err());
    }

    #[test]
    fn test_transform_requires_8_bytes() -> Result<(), micro_rpc::Status> {
        let mut service = create_sum_service();
        service.configure_and_attest(ConfigureAndAttestRequest::default())?;
        for length in (0..7).chain(9..16) {
            let request = TransformRequest {
                inputs: vec![encode_unencrypted(&vec![0; length])],
                ..Default::default()
            };
            assert!(service.transform(request).is_err());
        }
        Ok(())
    }

    #[test]
    fn test_transform_overflow() -> Result<(), micro_rpc::Status> {
        let mut service = create_sum_service();
        service.configure_and_attest(ConfigureAndAttestRequest::default())?;
        let request = TransformRequest {
            inputs: vec![encode_unencrypted(&[0, 0, 0, 0, 0, 0, 0, 0xFF]); 2],
            ..Default::default()
        };
        assert!(service.transform(request).is_err());
        Ok(())
    }

    #[test]
    fn test_transform_sums_inputs() -> Result<(), micro_rpc::Status> {
        let mut service = create_sum_service();
        service.configure_and_attest(ConfigureAndAttestRequest::default())?;
        for count in 0..10 {
            let request = TransformRequest {
                inputs: (1..(count + 1))
                    .map(|i| encode_unencrypted(&[0, i, 0, 0, 0, 0, 0, 0]))
                    .collect(),
                ..Default::default()
            };
            let expected = encode_unencrypted(&[0, (1..(count + 1)).sum(), 0, 0, 0, 0, 0, 0]);
            assert_eq!(
                service.transform(request)?,
                TransformResponse { outputs: vec![expected], ..Default::default() }
            );
        }
        Ok(())
    }

    #[test]
    fn test_transform_encrypted() -> Result<(), micro_rpc::Status> {
        let mut service = create_sum_service();
        let configure_response =
            service.configure_and_attest(ConfigureAndAttestRequest::default())?;
        let nonces_response = service.generate_nonces(GenerateNoncesRequest { nonces_count: 2 })?;
        let associated_data = b"associated data";

        let request = TransformRequest {
            inputs: vec![
                pipeline_transforms::io::create_rewrapped_record(
                    &[1, 0, 0, 0, 0, 0, 0, 0],
                    associated_data,
                    &configure_response.public_key,
                    &nonces_response.nonces[0],
                )
                .unwrap()
                .0,
                pipeline_transforms::io::create_rewrapped_record(
                    &[2, 0, 0, 0, 0, 0, 0, 0],
                    associated_data,
                    &configure_response.public_key,
                    &nonces_response.nonces[1],
                )
                .unwrap()
                .0,
            ],
            ..Default::default()
        };
        let expected = encode_unencrypted(&[3, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            service.transform(request)?,
            TransformResponse { outputs: vec![expected], ..Default::default() }
        );
        Ok(())
    }
}

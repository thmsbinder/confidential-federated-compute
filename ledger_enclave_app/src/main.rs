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
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::boxed::Box;
use oak_restricted_kernel_sdk::{
    attestation::InstanceEvidenceProvider,
    channel::{start_blocking_server, FileDescriptorChannel},
    crypto::InstanceSigner,
    entrypoint,
    utils::samplestore::StaticSampleStore,
};

#[entrypoint]
fn run_server() -> ! {
    // Logging from the restricted kernel is very slow (multiple milliseconds);
    // suppress low-priority messages in non-debug builds.
    #[cfg(not(debug_assertions))]
    {
        oak_restricted_kernel_sdk::utils::log::set_max_level(
            oak_restricted_kernel_sdk::utils::log::LevelFilter::Warn,
        );
    }

    let mut invocation_stats = StaticSampleStore::<1000>::new().unwrap();
    let service = ledger_service::LedgerService::create(
        Box::new(InstanceEvidenceProvider::create().unwrap()),
        Box::new(InstanceSigner::create().unwrap()),
    )
    .expect("failed to create LedgerService");
    let server = federated_compute::proto::LedgerServer::new(service);
    start_blocking_server(Box::<FileDescriptorChannel>::default(), server, &mut invocation_stats)
        .expect("server encountered an unrecoverable error");
}

// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;

pub mod v1 {
	use super::*;
	use frame_support::traits::GetStorageVersion;
	use frame_support::traits::OnRuntimeUpgrade;

	use sp_core::Get;

	pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
		fn on_runtime_upgrade() -> Weight {
			let current = Pallet::<T>::current_storage_version();
			let onchain = Pallet::<T>::on_chain_storage_version();

			if current == 1 && onchain == 0 {
				log!(
					info,
					"Running V1 migration with current storage version {:?} / onchain {:?}",
					current,
					onchain
				);

				current.put::<Pallet<T>>();
				T::DbWeight::get().reads_writes(2, 1)
			} else {
				log!(
					info,
					"Migration did not executed. This probably should be removed"
				);
				T::DbWeight::get().reads(2)
			}
		}
	}
}

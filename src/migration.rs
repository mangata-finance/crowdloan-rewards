
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
	use frame_support::pallet_prelude::*;
	use frame_support::traits::OnRuntimeUpgrade;
	use frame_support::dispatch::GetStorageVersion;

	use sp_core::Get;
	

	#[frame_support::storage_alias]
	pub(super) type UnassociatedContributions<T: Config> =
		StorageMap<Pallet<T>, Blake2_128Concat, <T as Config>::RelayChainAccountId, RewardInfo<T>>;

	#[frame_support::storage_alias]
	pub(super) type AccountsPayable<T: Config> =
		StorageMap<Pallet<T>, Blake2_128Concat, <T as frame_system::Config>::AccountId, RewardInfo<T>>;

	#[frame_support::storage_alias]
	pub type CrowdloanAllocation<T: Config> = StorageValue<Pallet<T>, mangata_types::Balance, ValueQuery>;

	#[frame_support::storage_alias]
	pub type ClaimedRelayChainIds<T: Config> = StorageMap<Pallet<T>, Blake2_128Concat, <T as Config>::RelayChainAccountId, ()>;

	#[frame_support::storage_alias]
	type InitRelayBlock<T: Config> = StorageValue<Pallet<T>, <T as Config>::VestingBlockNumber, ValueQuery>;

	#[frame_support::storage_alias]
	type EndRelayBlock<T: Config> = StorageValue<Pallet<T>, <T as Config>::VestingBlockNumber, ValueQuery>;
	
	/// Trivial migration which makes the roles of each pool optional.
	///
	/// Note: The depositor is not optional since he can never change.
	pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
		fn on_runtime_upgrade() -> Weight {
			let current = Pallet::<T>::current_storage_version();
			let onchain = Pallet::<T>::on_chain_storage_version();
			const DEFAULT_CROWDLOAN_ID: u32  = 0u32;
			let mut counter: u64 = 0;


			if current == 1 && onchain == 0 {
				log!(
					info,
					"Running V1 migration with current storage version {:?} / onchain {:?}",
					current,
					onchain
					);


				for (key, value) in v1::UnassociatedContributions::<T>::drain() {
					counter+=1;
					crate::pallet::UnassociatedContributions::<T>::insert(
						DEFAULT_CROWDLOAN_ID,
						key,
						value
						);
				}

				for (key, value) in v1::AccountsPayable::<T>::drain() {
					counter+=1;
					crate::pallet::AccountsPayable::<T>::insert(
						DEFAULT_CROWDLOAN_ID,
						key,
						value
						);
				}

				let mut writes_counter: u64 = 0;
				// When 1st crowdloan was initialized with all the data (relay & local ids set)
				// The `ClaimedRelayChainIds` storage entry is only used by `associate_native_identity`
				// when someone (root/council) would like to associate unassociated rewards with
				// particular account, so its obsolete to migrate this info. And because of
				// extra cost of migration of `ClaimedRelayChainIds` we would be exceeding the block limits
				// lets just ignore it
				for (_key, _value) in v1::ClaimedRelayChainIds::<T>::drain() {
					writes_counter+=1;
				}

				crate::pallet::CrowdloanAllocation::<T>::insert(
					DEFAULT_CROWDLOAN_ID,
					v1::CrowdloanAllocation::<T>::take(),
				);


				let init = v1::InitRelayBlock::<T>::take();
				let end = v1::EndRelayBlock::<T>::take();
				crate::pallet::CrowdloanPeriod::<T>::insert(DEFAULT_CROWDLOAN_ID, (init, end));

				log!(info, "Migrated entries: {}", counter);
				T::DbWeight::get().reads_writes(counter, writes_counter + counter + 2)
			} else {
				log!(info, "Migration did not executed. This probably should be removed");
				T::DbWeight::get().reads(2)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
			log!(info, "Crowdloan::pre_upgrade start");

			assert_eq!(Pallet::<T>::on_chain_storage_version(), 0);
			assert!(v1::AccountsPayable::<T>::iter().count() > 0);
			assert!(crate::pallet::CrowdloanPeriod::<T>::iter().count() == 0);

			log!(info, "Crowdloan::pre_upgrade finished");
			Ok(Default::default())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
			const DEFAULT_CROWDLOAN_ID: u32  = 0u32;
			log!(info, "Crowdlon::post_upgrade start");
			assert_eq!(Pallet::<T>::current_storage_version(), 1);
			
			for (key, key2, value) in crate::pallet::AccountsPayable::<T>::iter() {
				assert_eq!(key, DEFAULT_CROWDLOAN_ID);
			}

			assert!(crate::AccountsPayable::<T>::iter().count() > 0);

			log!(info, "Crowdlon::post_upgrade finished");
			Ok(())
		}
	}
}


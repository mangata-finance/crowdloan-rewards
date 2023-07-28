
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


			if current == 1 && onchain == 0 {
				log!(
					info,
					"Running V1 migration with current storage version {:?} / onchain {:?}",
					current,
					onchain
					);

				for (key, value) in v1::UnassociatedContributions::<T>::drain() {
					crate::pallet::UnassociatedContributions::<T>::insert(
						DEFAULT_CROWDLOAN_ID,
						key,
						value
						);
				}

				for (key, value) in v1::AccountsPayable::<T>::drain() {
					crate::pallet::AccountsPayable::<T>::insert(
						DEFAULT_CROWDLOAN_ID,
						key,
						value
						);
				}


				for (key, value) in v1::ClaimedRelayChainIds::<T>::drain() {
					crate::pallet::ClaimedRelayChainIds::<T>::insert(
						DEFAULT_CROWDLOAN_ID,
						key,
						value
						);
				}

				crate::pallet::CrowdloanAllocation::<T>::insert(
					DEFAULT_CROWDLOAN_ID,
					v1::CrowdloanAllocation::<T>::take(),
					);


				let init = v1::InitRelayBlock::<T>::take();
				let end = v1::EndRelayBlock::<T>::take();
				crate::pallet::CrowdloanPeriod::<T>::insert(DEFAULT_CROWDLOAN_ID, (init, end));

				T::DbWeight::get().reads(2)
			} else {
				log!(info, "Migration did not executed. This probably should be removed");
				T::DbWeight::get().reads(2)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<(), &'static str> {
			log!(info, "Crowdloan::pre_upgrade");

			assert_eq!(Pallet::<T>::on_chain_storage_version(), 0);
			assert!(v1::AccountsPayable::<T>::iter().count() > 0);

			assert!(crate::pallet::AccountsPayable::<T>::iter().count() == 0);
			assert!(crate::pallet::CrowdloanPeriod::<T>::iter().count()	== 0);
			Ok(())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade() -> Result<(), &'static str> {
			log!(info, "Crowdloan::post_upgrade");

			assert_eq!(Pallet::<T>::on_chain_storage_version(), 1);
			assert!(v1::AccountsPayable::<T>::iter().count() == 0);

			assert!(crate::pallet::AccountsPayable::<T>::iter().count() > 0);
			assert!(crate::pallet::CrowdloanPeriod::<T>::iter().count()	== 1);

			Ok(())
		}
	}
}


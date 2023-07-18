// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! Autogenerated weights for pallet_crowdloan_rewards
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-08-01, STEPS: `[32, ]`, REPEAT: 64, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// ./target/release/moonbeam
// benchmark
// --chain
// dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet
// pallet_crowdloan_rewards
// --extrinsic
// *
// --steps
// 32
// --repeat
// 64
// --raw
// --template=./benchmarking/frame-weight-template.hbs
// --output
// /tmp/

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_crowdloan_rewards.
pub trait WeightInfo {
	fn initialize_reward_vec(x: u32) -> Weight;
	fn complete_initialization() -> Weight;
	fn claim() -> Weight;
	fn update_reward_address() -> Weight;
	fn associate_native_identity() -> Weight;
	fn change_association_with_relay_keys(x: u32) -> Weight;
	fn set_crowdloan_allocation() -> Weight;
}

/// Weights for pallet_crowdloan_rewards using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn initialize_reward_vec(x: u32) -> Weight {
		Weight::from_ref_time(143_109_000)
			// Standard Error: 21_000
			.saturating_add(Weight::from_ref_time(
				72_298_000_u64.saturating_mul(x as u64),
			))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().reads(x as u64))
			.saturating_add(T::DbWeight::get().writes(5))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	fn complete_initialization() -> Weight {
		Weight::from_ref_time(51_047_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn set_crowdloan_allocation() -> Weight {
		Weight::from_ref_time(147_000).saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	fn claim() -> Weight {
		Weight::from_ref_time(101_484_000)
			.saturating_add(T::DbWeight::get().reads(11))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	fn update_reward_address() -> Weight {
		Weight::from_ref_time(59_051_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn associate_native_identity() -> Weight {
		Weight::from_ref_time(152_997_000)
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn change_association_with_relay_keys(x: u32) -> Weight {
		Weight::from_ref_time(0)
			// Standard Error: 7_000
			.saturating_add(Weight::from_ref_time(
				47_373_000_u64.saturating_mul(x as u64),
			))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(4))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn initialize_reward_vec(x: u32) -> Weight {
		Weight::from_ref_time(143_109_000)
			// Standard Error: 21_000
			.saturating_add(Weight::from_ref_time(
				72_298_000_u64.saturating_mul(x as u64),
			))
			.saturating_add(RocksDbWeight::get().reads(8))
			.saturating_add(RocksDbWeight::get().reads(x as u64))
			.saturating_add(RocksDbWeight::get().writes(5))
			.saturating_add(RocksDbWeight::get().writes(x as u64))
	}
	fn complete_initialization() -> Weight {
		Weight::from_ref_time(51_047_000)
			.saturating_add(RocksDbWeight::get().reads(6))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	fn claim() -> Weight {
		Weight::from_ref_time(101_484_000)
			.saturating_add(RocksDbWeight::get().reads(11))
			.saturating_add(RocksDbWeight::get().writes(5))
	}
	fn update_reward_address() -> Weight {
		Weight::from_ref_time(59_051_000)
			.saturating_add(RocksDbWeight::get().reads(6))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	fn associate_native_identity() -> Weight {
		Weight::from_ref_time(152_997_000)
			.saturating_add(RocksDbWeight::get().reads(9))
			.saturating_add(RocksDbWeight::get().writes(7))
	}
	fn change_association_with_relay_keys(x: u32) -> Weight {
		Weight::from_ref_time(0)
			// Standard Error: 7_000
			.saturating_add(Weight::from_ref_time(
				47_373_000_u64.saturating_mul(x as u64),
			))
			.saturating_add(RocksDbWeight::get().reads(6))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	fn set_crowdloan_allocation() -> Weight {
		Weight::from_ref_time(147_000).saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
}

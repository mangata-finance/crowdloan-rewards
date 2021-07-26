// Copyright 2019-2021 PureStake Inc.
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
//! DATE: 2021-07-26, STEPS: `[32, ]`, REPEAT: 64, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
}

/// Weights for pallet_crowdloan_rewards using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn initialize_reward_vec(x: u32) -> Weight {
		(66_138_000 as Weight)
			// Standard Error: 14_000
			.saturating_add((72_994_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().reads((4 as Weight).saturating_mul(x as Weight)))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
			.saturating_add(T::DbWeight::get().writes((3 as Weight).saturating_mul(x as Weight)))
	}
	fn complete_initialization() -> Weight {
		(43_160_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn claim() -> Weight {
		(102_313_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(11 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	fn update_reward_address() -> Weight {
		(58_552_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn associate_native_identity() -> Weight {
		(154_280_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn initialize_reward_vec(x: u32) -> Weight {
		(66_138_000 as Weight)
			// Standard Error: 14_000
			.saturating_add((72_994_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(RocksDbWeight::get().reads(8 as Weight))
			.saturating_add(RocksDbWeight::get().reads((4 as Weight).saturating_mul(x as Weight)))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
			.saturating_add(RocksDbWeight::get().writes((3 as Weight).saturating_mul(x as Weight)))
	}
	fn complete_initialization() -> Weight {
		(43_160_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(4 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	fn claim() -> Weight {
		(102_313_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(11 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
	}
	fn update_reward_address() -> Weight {
		(58_552_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(6 as Weight))
			.saturating_add(RocksDbWeight::get().writes(4 as Weight))
	}
	fn associate_native_identity() -> Weight {
		(154_280_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(8 as Weight))
			.saturating_add(RocksDbWeight::get().writes(7 as Weight))
	}
}

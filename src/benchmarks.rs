#![cfg(feature = "runtime-benchmarks")]

use crate::Config;
use crate::{Call, Pallet, WRAPPED_BYTES_POSTFIX, WRAPPED_BYTES_PREFIX};
use ed25519_dalek::Signer;
use frame_benchmarking::Zero;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::assert_ok;
use frame_support::traits::{Currency, Get, OnFinalize};
use frame_system::RawOrigin;
use mangata_types::Balance;
use orml_tokens::MultiTokenCurrencyExtended;
use parity_scale_codec::Encode;
use sp_core::{
	crypto::{AccountId32, UncheckedFrom},
	ed25519,
};
use sp_runtime::{
	traits::{BlockNumberProvider, One},
	MultiSignature,
};
use sp_std::vec;
use sp_std::vec::Vec;

/// Default balance amount is minimum contribution
fn default_balance<T: Config>() -> Balance {
	T::MinimumReward::get()
}

/// Create a funded user.
fn fund_specific_account<T: Config>(pallet_account: T::AccountId, extra: Balance) {
	let default_balance = default_balance::<T>();
	let total = default_balance + extra;
	// TODO: fix
	// T::RewardCurrency::make_free_balance_be(&pallet_account, total);
	// T::RewardCurrency::issue(total);
}

/// Create a funded user.
fn create_funded_user<T: Config>(string: &'static str, n: u32, extra: Balance) -> T::AccountId {
	const SEED: u32 = 0;
	let user = account(string, n, SEED);
	let default_balance = default_balance::<T>();
	let total = default_balance + extra;
	while T::Tokens::get_next_currency_id().into() <= T::NativeTokenId::get() {
		T::Tokens::create(&user, 0u32.into()).unwrap();
	}
	assert_ok!(T::Tokens::mint(
		T::NativeTokenId::get().into(),
		&user,
		total.into()
	));
	user
}

/// Create contributors.
fn create_contributors<T: Config>(
	total_number: u32,
	seed_offset: u32,
) -> Vec<(T::RelayChainAccountId, Option<T::AccountId>, Balance)> {
	let mut contribution_vec = Vec::new();
	for i in 0..total_number {
		let seed = SEED - seed_offset - i;
		let mut account: [u8; 32] = [0u8; 32];
		let seed_as_slice = seed.to_be_bytes();
		for j in 0..seed_as_slice.len() {
			account[j] = seed_as_slice[j]
		}
		let relay_chain_account: AccountId32 = account.into();
		let user = create_funded_user::<T>("user", seed, 0u32.into());
		let contribution: Balance = 100u32.into();
		contribution_vec.push((relay_chain_account.into(), Some(user.clone()), contribution));
	}
	contribution_vec
}

/// Insert contributors.
fn insert_contributors<T: Config>(
	contributors: Vec<(T::RelayChainAccountId, Option<T::AccountId>, Balance)>,
) -> Result<(), &'static str> {
	let mut sub_vec = Vec::new();
	let batch = max_batch_contributors::<T>();
	// Due to the MaxInitContributors associated type, we need ton insert them in batches
	// When we reach the batch size, we insert them
	let amount = contributors
		.iter()
		.fold(0, |acc, (_, _, amount)| acc + amount);
	Pallet::<T>::set_crowdloan_allocation(RawOrigin::Root.into(), amount)?;

	for i in 0..contributors.len() {
		sub_vec.push(contributors[i].clone());
		// If we reached the batch size, we should insert them
		if i as u32 % batch == batch - 1 || i == contributors.len() - 1 {
			Pallet::<T>::initialize_reward_vec(RawOrigin::Root.into(), sub_vec.clone())?;
			sub_vec.clear()
		}
	}
	Ok(())
}

/// Create a Contributor.
fn close_initialization<T: Config>(
	init_relay: T::VestingBlockNumber,
	end_relay: T::VestingBlockNumber,
) -> Result<(), &'static str> {
	Pallet::<T>::complete_initialization(RawOrigin::Root.into(), init_relay, end_relay)?;
	Ok(())
}

fn create_sig<T: Config>(seed: u32, payload: Vec<u8>) -> (AccountId32, MultiSignature) {
	// Crate seed
	let mut seed_32: [u8; 32] = [0u8; 32];
	let seed_as_slice = seed.to_be_bytes();
	for j in 0..seed_as_slice.len() {
		seed_32[j] = seed_as_slice[j]
	}

	let secret = ed25519_dalek::SecretKey::from_bytes(&seed_32).unwrap();
	let public = ed25519_dalek::PublicKey::from(&secret);
	let pair = ed25519_dalek::Keypair { secret, public };
	let sig = pair.sign(&payload).to_bytes();
	let signature: MultiSignature = ed25519::Signature::from_raw(sig).into();

	let ed_public: ed25519::Public = ed25519::Public::unchecked_from(public.to_bytes());
	let account: AccountId32 = ed_public.into();
	(account, signature.into())
}

fn max_batch_contributors<T: Config>() -> u32 {
	<<T as Config>::MaxInitContributors as Get<u32>>::get()
}

// This is our current number of contributors
const MAX_ALREADY_USERS: u32 = 5799;
const SEED: u32 = 999999999;

benchmarks! {
	set_crowdloan_allocation{
		assert!(Pallet::<T>::get_crowdloan_allocation(0u32).is_zero());

	}:  _(RawOrigin::Root, 10000u128)
	verify {
		assert_eq!(Pallet::<T>::get_crowdloan_allocation(0u32), 10000u128);
	}
	initialize_reward_vec {
		let x in 1..max_batch_contributors::<T>();
		let y = MAX_ALREADY_USERS;



		// Create y contributors
		let contributors = create_contributors::<T>(y, 0);

		// Insert them
		let amount = contributors.iter().fold(0, |acc, (_,_,amount)| acc + amount);
		assert_ok!(Pallet::<T>::set_crowdloan_allocation(RawOrigin::Root.into(), amount));
		insert_contributors::<T>(contributors)?;

		// This X new contributors are the ones we will count
		let new_contributors = create_contributors::<T>(x, y);
		let new_amount = new_contributors.iter().fold(0, |acc, (_,_,amount)| acc + amount);
		assert_ok!(Pallet::<T>::set_crowdloan_allocation(RawOrigin::Root.into(), amount + new_amount));

		let verifier = create_funded_user::<T>("user", SEED, 0u32.into());

	}:  _(RawOrigin::Root, new_contributors)
	verify {
		assert!(Pallet::<T>::accounts_payable(0u32, &verifier).is_some());
	}

	complete_initialization {
		// Fund pallet account
		let total_pot = 100u32;
		assert_ok!(Pallet::<T>::set_crowdloan_allocation(RawOrigin::Root.into(), total_pot.into()));
		// 1 contributor is enough
		let contributors = create_contributors::<T>(1, 0);

		// Insert them
		insert_contributors::<T>(contributors)?;

		// We need to create the first block inherent, to initialize the initRelayBlock
		T::VestingBlockProvider::set_block_number(1u32.into());
		Pallet::<T>::on_finalize(T::BlockNumber::one());

	}:  _(RawOrigin::Root, 1u32.into(), 10u32.into())
	verify {
	  assert!(Pallet::<T>::initialized());
	}

	claim {
		// Fund pallet account
		let total_pot = 100u32;
		assert_ok!(Pallet::<T>::set_crowdloan_allocation(RawOrigin::Root.into(), total_pot.into()));

		// The user that will make the call
		let caller: T::AccountId = create_funded_user::<T>("user", SEED, 100u32.into());

		// We verified there is no dependency of the number of contributors already inserted in claim
		// Create 1 contributor
		let contributors: Vec<(T::RelayChainAccountId, Option<T::AccountId>, Balance)> =
			vec![(AccountId32::from([1u8;32]).into(), Some(caller.clone()), total_pot.into())];

		// Insert them
		insert_contributors::<T>(contributors)?;

		// Close initialization
		close_initialization::<T>(1u32.into(), 10u32.into())?;

		// First inherent
		T::VestingBlockProvider::set_block_number(1u32.into());
		Pallet::<T>::on_finalize(T::BlockNumber::one());

		// Claimed
		let claimed_reward = Pallet::<T>::accounts_payable(0u32, &caller).unwrap().claimed_reward;

		// Create 4th relay block, by now the user should have vested some amount
		T::VestingBlockProvider::set_block_number(4u32.into());
	}:  _(RawOrigin::Signed(caller.clone()),None)
	verify {
		assert!(Pallet::<T>::accounts_payable(0u32, &caller).unwrap().claimed_reward > claimed_reward);
	}

	update_reward_address {
		// Fund pallet account
		let total_pot = 100u32;
		assert_ok!(Pallet::<T>::set_crowdloan_allocation(RawOrigin::Root.into(), total_pot.into()));

		// The user that will make the call
		let caller: T::AccountId = create_funded_user::<T>("user", SEED, 100u32.into());

		let relay_account: T::RelayChainAccountId = AccountId32::from([1u8;32]).into();
		// We verified there is no dependency of the number of contributors already inserted in update_reward_address
		// Create 1 contributor
		let contributors: Vec<(T::RelayChainAccountId, Option<T::AccountId>, Balance)> =
			vec![(relay_account.clone(), Some(caller.clone()), total_pot.into())];

		// Insert them
		insert_contributors::<T>(contributors)?;

		// Close initialization
		close_initialization::<T>(1u32.into(), 10u32.into())?;

		// First inherent
		T::VestingBlockProvider::set_block_number(1u32.into());
		Pallet::<T>::on_finalize(T::BlockNumber::one());


		// Let's advance the relay so that the vested  amount get transferred
		T::VestingBlockProvider::set_block_number(4u32.into());

		// The new user
		let new_user = create_funded_user::<T>("user", SEED+1, 0u32.into());

	}:  _(RawOrigin::Signed(caller.clone()), new_user.clone(), None)
	verify {
		assert_eq!(Pallet::<T>::accounts_payable(0u32, &new_user).unwrap().total_reward, (100u32.into()));
		assert!(Pallet::<T>::claimed_relay_chain_ids(0u32, &relay_account).is_some());
	}

	associate_native_identity {
		// Fund pallet account
		let total_pot = 100u32;
		// fund_specific_account::<T>(Pallet::<T>::account_id(), total_pot.into());

		// The caller that will associate the account
		let caller: T::AccountId = create_funded_user::<T>("user", SEED, 100u32.into());

		// Construct payload
		let mut payload = WRAPPED_BYTES_PREFIX.to_vec();
		payload.append(&mut T::SignatureNetworkIdentifier::get().to_vec());
		payload.append(&mut caller.clone().encode());
		payload.append(&mut WRAPPED_BYTES_POSTFIX.to_vec());

		// Create a fake sig for such an account
		let (relay_account, signature) = create_sig::<T>(SEED, payload);

		// We verified there is no dependency of the number of contributors already inserted in associate_native_identity
		// Create 1 contributor
		let contributors: Vec<(T::RelayChainAccountId, Option<T::AccountId>, Balance)> =
		vec![(relay_account.clone().into(), None, total_pot.into())];

		// Insert them
		insert_contributors::<T>(contributors)?;

		// Clonse initialization
		close_initialization::<T>(1u32.into(), 10u32.into())?;

		// First inherent
		T::VestingBlockProvider::set_block_number(1u32.into());
		Pallet::<T>::on_finalize(T::BlockNumber::one());

	}:  _(RawOrigin::Root, caller.clone(), relay_account.into(), signature)
	verify {
		assert_eq!(Pallet::<T>::accounts_payable(0u32, &caller).unwrap().total_reward, (100u32.into()));
	}

	change_association_with_relay_keys {

		// The weight will depend on the number of proofs provided
		// We need to parameterize this value
		// We leave this as the max batch length
		let x in 1..max_batch_contributors::<T>();

		// Fund pallet account
		let total_pot = 100u32*x;
		assert_ok!(Pallet::<T>::set_crowdloan_allocation(RawOrigin::Root.into(), total_pot.into()));

		// The first reward account that will associate the account
		let first_reward_account: T::AccountId = create_funded_user::<T>("user", SEED, 100u32.into());

		// The account to which we will update our reward account
		let second_reward_account: T::AccountId = create_funded_user::<T>("user", SEED-1, 100u32.into());

		let mut proofs: Vec<(T::RelayChainAccountId, MultiSignature)> = Vec::new();

		// Construct payload
		let mut payload = WRAPPED_BYTES_PREFIX.to_vec();
		payload.append(&mut T::SignatureNetworkIdentifier::get().to_vec());
		payload.append(&mut second_reward_account.clone().encode());
		payload.append(&mut first_reward_account.clone().encode());
		payload.append(&mut WRAPPED_BYTES_POSTFIX.to_vec());

		// Create N sigs for N accounts
		for i in 0..x {
			let (relay_account, signature) = create_sig::<T>(SEED-i, payload.clone());
			proofs.push((relay_account.into(), signature));
		}

		// Create x contributors
		// All of them map to the same account
		let mut contributors: Vec<(T::RelayChainAccountId, Option<T::AccountId>, Balance)> = Vec::new();
		for (relay_account, _) in proofs.clone() {
			contributors.push((relay_account, Some(first_reward_account.clone()), 100u32.into()));
		}

		// Insert them
		insert_contributors::<T>(contributors.clone())?;

		// Clonse initialization
		close_initialization::<T>(1u32.into(), 10u32.into())?;

		// First inherent
		T::VestingBlockProvider::set_block_number(1u32.into());
		Pallet::<T>::on_finalize(T::BlockNumber::one());

	}:  _(RawOrigin::Root, second_reward_account.clone(), first_reward_account.clone(), proofs)
	verify {
		assert!(Pallet::<T>::accounts_payable(0u32, &second_reward_account).is_some());
		assert_eq!(Pallet::<T>::accounts_payable(0u32, &second_reward_account).unwrap().total_reward, (100u32*x).into());
		assert!(Pallet::<T>::accounts_payable(0u32, &first_reward_account).is_none());

	}

}
#[cfg(test)]
mod tests {
	use crate::mock::Test;
	use sp_io::TestExternalities;

	pub fn new_test_ext() -> TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		TestExternalities::new(t)
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::benchmarks::tests::new_test_ext(),
	crate::mock::Test
);

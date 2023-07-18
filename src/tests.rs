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

//! Unit testing

use crate::*;
use frame_support::dispatch::{DispatchError, Dispatchable};
use frame_support::traits::tokens::currency::MultiTokenCurrency;
use frame_support::traits::WithdrawReasons;
use frame_support::{assert_noop, assert_ok};
use mangata_types::TokenId;
use mock::*;
use pallet_vesting_mangata::MultiTokenVestingSchedule;
use parity_scale_codec::Encode;
use sp_core::Pair;
use sp_runtime::{ModuleError, MultiSignature};

type TokensOf<Test> = <Test as Config>::Tokens;

// Constant that reflects the desired vesting period for the tests
// Most tests complete initialization passing initRelayBlock + VESTING as the endRelayBlock
const VESTING: u64 = 8;

fn transferable_balance(who: &u64, token_id: TokenId) -> u128 {
	TokensOf::<Test>::free_balance(token_id, who) - Vesting::vesting_balance(who, token_id).unwrap()
}

#[test]
fn geneses() {
	empty().execute_with(|| {
		assert!(System::events().is_empty());
		// Insert contributors
		let pairs = get_ed25519_pairs(3);
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 32].into(), Some(1), 500u32.into()),
				([2u8; 32].into(), Some(2), 500u32.into()),
				(pairs[0].public().into(), None, 500u32.into()),
				(pairs[1].public().into(), None, 500u32.into()),
				(pairs[2].public().into(), None, 500u32.into())
			]
		));

		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));
		assert_eq!(Crowdloan::total_contributors(), 5);

		// accounts_payable
		assert!(Crowdloan::accounts_payable(&1).is_some());
		assert!(Crowdloan::accounts_payable(&2).is_some());
		assert!(Crowdloan::accounts_payable(&3).is_none());
		assert!(Crowdloan::accounts_payable(&4).is_none());
		assert!(Crowdloan::accounts_payable(&5).is_none());

		// claimed address existence
		assert!(Crowdloan::claimed_relay_chain_ids(&[1u8; 32]).is_some());
		assert!(Crowdloan::claimed_relay_chain_ids(&[2u8; 32]).is_some());
		assert!(Crowdloan::claimed_relay_chain_ids(pairs[0].public().as_array_ref()).is_none());
		assert!(Crowdloan::claimed_relay_chain_ids(pairs[1].public().as_array_ref()).is_none());
		assert!(Crowdloan::claimed_relay_chain_ids(pairs[2].public().as_array_ref()).is_none());

		// unassociated_contributions
		assert!(Crowdloan::unassociated_contributions(&[1u8; 32]).is_none());
		assert!(Crowdloan::unassociated_contributions(&[2u8; 32]).is_none());
		assert!(Crowdloan::unassociated_contributions(pairs[0].public().as_array_ref()).is_some());
		assert!(Crowdloan::unassociated_contributions(pairs[1].public().as_array_ref()).is_some());
		assert!(Crowdloan::unassociated_contributions(pairs[2].public().as_array_ref()).is_some());
	});
}

#[test]
fn proving_assignation_works() {
	let pairs = get_ed25519_pairs(3);
	let mut payload = WRAPPED_BYTES_PREFIX.to_vec();
	payload.append(&mut TestSigantureNetworkIdentifier::get().to_vec());
	payload.append(&mut 3u64.encode());
	payload.append(&mut WRAPPED_BYTES_POSTFIX.to_vec());
	let signature: MultiSignature = pairs[0].sign(&payload).into();
	let alread_associated_signature: MultiSignature = pairs[0].sign(&1u64.encode()).into();
	empty().execute_with(|| {
		// Insert contributors
		let pairs = get_ed25519_pairs(3);
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 32].into(), Some(1), 500u32.into()),
				([2u8; 32].into(), Some(2), 500u32.into()),
				(pairs[0].public().into(), None, 500u32.into()),
				(pairs[1].public().into(), None, 500u32.into()),
				(pairs[2].public().into(), None, 500u32.into())
			],
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));
		// 4 is not payable first
		assert!(Crowdloan::accounts_payable(&3).is_none());
		assert_eq!(
			Crowdloan::accounts_payable(&1)
				.unwrap()
				.contributed_relay_addresses,
			vec![[1u8; 32]]
		);

		roll_to(4);
		// Signature is wrong, prove fails
		assert_noop!(
			Crowdloan::associate_native_identity(
				RuntimeOrigin::signed(4),
				4,
				pairs[0].public().into(),
				signature.clone()
			),
			Error::<Test>::InvalidClaimSignature
		);

		// Signature is right, but address already claimed
		assert_noop!(
			Crowdloan::associate_native_identity(
				RuntimeOrigin::signed(4),
				1,
				pairs[0].public().into(),
				alread_associated_signature
			),
			Error::<Test>::AlreadyAssociated
		);

		// Signature is right, prove passes
		assert_ok!(Crowdloan::associate_native_identity(
			RuntimeOrigin::signed(4),
			3,
			pairs[0].public().into(),
			signature.clone()
		));

		// Signature is right, but relay address is no longer on unassociated
		assert_noop!(
			Crowdloan::associate_native_identity(
				RuntimeOrigin::signed(4),
				3,
				pairs[0].public().into(),
				signature
			),
			Error::<Test>::NoAssociatedClaim
		);

		// now three is payable
		assert!(Crowdloan::accounts_payable(&3).is_some());
		assert_eq!(
			Crowdloan::accounts_payable(&3)
				.unwrap()
				.contributed_relay_addresses,
			vec![*pairs[0].public().as_array_ref()]
		);

		assert!(Crowdloan::unassociated_contributions(pairs[0].public().as_array_ref()).is_none());
		assert!(Crowdloan::claimed_relay_chain_ids(pairs[0].public().as_array_ref()).is_some());

		let expected = vec![crate::Event::NativeIdentityAssociated(
			pairs[0].public().into(),
			3,
			500,
		)];
		assert_eq!(events(), expected);
	});
}

#[test]
fn initializing_multi_relay_to_single_native_address_works() {
	empty().execute_with(|| {
		// Insert contributors
		let pairs = get_ed25519_pairs(3);
		// The init relay block gets inserted
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 32].into(), Some(1), 500u32.into()),
				([2u8; 32].into(), Some(1), 500u32.into()),
				(pairs[0].public().into(), None, 500u32.into()),
				(pairs[1].public().into(), None, 500u32.into()),
				(pairs[2].public().into(), None, 500u32.into())
			]
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));
		// 1 is payable
		assert!(Crowdloan::accounts_payable(&1).is_some());
		assert_eq!(
			Crowdloan::accounts_payable(&1)
				.unwrap()
				.contributed_relay_addresses,
			vec![[1u8; 32], [2u8; 32]]
		);

		roll_to(3);
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_eq!(
			Crowdloan::accounts_payable(&1).unwrap().claimed_reward,
			1000
		);
		assert_noop!(
			Crowdloan::claim(RuntimeOrigin::signed(3), None),
			Error::<Test>::NoAssociatedClaim
		);

		let expected = vec![crate::Event::RewardsPaid(1, 1000)];
		assert_eq!(events(), expected);
	});
}

#[test]
fn paying_works_step_by_step() {
	empty().execute_with(|| {
		// Insert contributors
		let pairs = get_ed25519_pairs(3);
		// The init relay block gets inserted
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 32].into(), Some(1), 500u32.into()),
				([2u8; 32].into(), Some(2), 500u32.into()),
				(pairs[0].public().into(), None, 500u32.into()),
				(pairs[1].public().into(), None, 500u32.into()),
				(pairs[2].public().into(), None, 500u32.into())
			]
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));
		// 1 is payable
		assert!(Crowdloan::accounts_payable(&1).is_some());
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_eq!(Crowdloan::accounts_payable(&1).unwrap().claimed_reward, 500);
		assert_eq!(transferable_balance(&1, 0), 100);

		roll_to(3);

		assert_eq!(transferable_balance(&1, 0), 200);
		assert_noop!(
			Crowdloan::claim(RuntimeOrigin::signed(3), None),
			Error::<Test>::NoAssociatedClaim
		);
		roll_to(4);
		assert_eq!(transferable_balance(&1, 0), 250);
		roll_to(5);
		assert_eq!(transferable_balance(&1, 0), 300);
		roll_to(6);
		assert_eq!(transferable_balance(&1, 0), 350);
		roll_to(7);
		assert_eq!(transferable_balance(&1, 0), 400);
		roll_to(8);
		assert_eq!(transferable_balance(&1, 0), 450);
		roll_to(9);
		assert_eq!(transferable_balance(&1, 0), 500);
		roll_to(10);
		assert_noop!(
			Crowdloan::claim(RuntimeOrigin::signed(1), None),
			Error::<Test>::RewardsAlreadyClaimed
		);

		let expected = vec![crate::Event::RewardsPaid(1, 500)];
		assert_eq!(events(), expected);
	});
}

#[test]
fn paying_works_after_unclaimed_period() {
	empty().execute_with(|| {
		// Insert contributors
		let pairs = get_ed25519_pairs(3);
		// The init relay block gets inserted
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 32].into(), Some(1), 500u32.into()),
				([2u8; 32].into(), Some(2), 500u32.into()),
				(pairs[0].public().into(), None, 500u32.into()),
				(pairs[1].public().into(), None, 500u32.into()),
				(pairs[2].public().into(), None, 500u32.into())
			]
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));

		// 1 is payable
		assert!(Crowdloan::accounts_payable(&1).is_some());
		roll_to(3);
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_eq!(Crowdloan::accounts_payable(&1).unwrap().claimed_reward, 500);
		assert_noop!(
			Crowdloan::claim(RuntimeOrigin::signed(3), None),
			Error::<Test>::NoAssociatedClaim
		);
		roll_to(4);
		// assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_eq!(transferable_balance(&1, 0), 250);
		roll_to(5);
		// assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_eq!(transferable_balance(&1, 0), 300);
		roll_to(6);
		// assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_eq!(transferable_balance(&1, 0), 350);
		roll_to(10);
		// assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_eq!(transferable_balance(&1, 0), 500);
		roll_to(329);
		assert_noop!(
			Crowdloan::claim(RuntimeOrigin::signed(1), None),
			Error::<Test>::RewardsAlreadyClaimed
		);

		let expected = vec![crate::Event::RewardsPaid(1, 500)];
		assert_eq!(events(), expected);
	});
}

#[test]
fn paying_late_joiner_works() {
	let pairs = get_ed25519_pairs(3);
	let mut payload = WRAPPED_BYTES_PREFIX.to_vec();
	payload.append(&mut TestSigantureNetworkIdentifier::get().to_vec());
	payload.append(&mut 3u64.encode());
	payload.append(&mut WRAPPED_BYTES_POSTFIX.to_vec());
	let signature: MultiSignature = pairs[0].sign(&payload).into();
	empty().execute_with(|| {
		// Insert contributors
		let pairs = get_ed25519_pairs(3);
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 32].into(), Some(1), 500u32.into()),
				([2u8; 32].into(), Some(2), 500u32.into()),
				(pairs[0].public().into(), None, 500u32.into()),
				(pairs[1].public().into(), None, 500u32.into()),
				(pairs[2].public().into(), None, 500u32.into())
			]
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));

		roll_to(12);
		assert_ok!(Crowdloan::associate_native_identity(
			RuntimeOrigin::signed(4),
			3,
			pairs[0].public().into(),
			signature.clone()
		));
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(3), None));
		assert_eq!(Crowdloan::accounts_payable(&3).unwrap().claimed_reward, 500);
		let expected = vec![
			crate::Event::NativeIdentityAssociated(pairs[0].public().into(), 3, 500),
			crate::Event::RewardsPaid(3, 500),
		];
		assert_eq!(events(), expected);
	});
}

#[test]
fn update_address_works() {
	empty().execute_with(|| {
		// Insert contributors
		let pairs = get_ed25519_pairs(3);
		// The init relay block gets inserted
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 32].into(), Some(1), 500u32.into()),
				([2u8; 32].into(), Some(2), 500u32.into()),
				(pairs[0].public().into(), None, 500u32.into()),
				(pairs[1].public().into(), None, 500u32.into()),
				(pairs[2].public().into(), None, 500u32.into())
			]
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));

		roll_to(3);
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_noop!(
			Crowdloan::claim(RuntimeOrigin::signed(8), None),
			Error::<Test>::NoAssociatedClaim
		);
		assert_ok!(Crowdloan::update_reward_address(
			RuntimeOrigin::signed(1),
			8
		));
		assert_eq!(transferable_balance(&1, 0), 200);
		roll_to(5);

		assert_eq!(transferable_balance(&1, 0), 300);
		// The initial payment is not
		let expected = vec![
			crate::Event::RewardsPaid(1, 500),
			crate::Event::RewardAddressUpdated(1, 8),
		];
		assert_eq!(events(), expected);
	});
}

#[test]
fn update_address_with_existing_address_fails() {
	empty().execute_with(|| {
		// Insert contributors
		let pairs = get_ed25519_pairs(3);
		// The init relay block gets inserted
		roll_to(2);
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 32].into(), Some(1), 500u32.into()),
				([2u8; 32].into(), Some(2), 500u32.into()),
				(pairs[0].public().into(), None, 500u32.into()),
				(pairs[1].public().into(), None, 500u32.into()),
				(pairs[2].public().into(), None, 500u32.into())
			]
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));

		roll_to(4);
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(2), None));
		assert_noop!(
			Crowdloan::update_reward_address(RuntimeOrigin::signed(1), 2),
			Error::<Test>::AlreadyAssociated
		);
	});
}

#[test]
fn update_address_with_existing_with_multi_address_works() {
	empty().execute_with(|| {
		// Insert contributors
		let pairs = get_ed25519_pairs(3);
		// The init relay block gets inserted
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 32].into(), Some(1), 500u32.into()),
				([2u8; 32].into(), Some(1), 500u32.into()),
				(pairs[0].public().into(), None, 500u32.into()),
				(pairs[1].public().into(), None, 500u32.into()),
				(pairs[2].public().into(), None, 500u32.into())
			]
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));

		roll_to(3);
		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(1), None));

		// We make sure all rewards go to the new address
		assert_ok!(Crowdloan::update_reward_address(
			RuntimeOrigin::signed(1),
			2
		));
		assert_eq!(
			Crowdloan::accounts_payable(&2).unwrap().claimed_reward,
			1000
		);
		assert_eq!(Crowdloan::accounts_payable(&2).unwrap().total_reward, 1000);

		assert_noop!(
			Crowdloan::claim(RuntimeOrigin::signed(1), None),
			Error::<Test>::NoAssociatedClaim
		);
	});
}

#[test]
fn initialize_new_addresses() {
	empty().execute_with(|| {
		// The init relay block gets inserted
		roll_to(2);
		// Insert contributors
		let pairs = get_ed25519_pairs(3);
		let init_block = 1u64;
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 32].into(), Some(1), 500u32.into()),
				([2u8; 32].into(), Some(2), 500u32.into()),
				(pairs[0].public().into(), None, 500u32.into()),
				(pairs[1].public().into(), None, 500u32.into()),
				(pairs[2].public().into(), None, 500u32.into())
			]
		));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));

		assert_eq!(Crowdloan::initialized(), true);

		roll_to(4);
		assert_noop!(
			Crowdloan::initialize_reward_vec(
				RuntimeOrigin::root(),
				vec![([1u8; 32].into(), Some(1), 500u32.into())]
			),
			Error::<Test>::RewardVecAlreadyInitialized,
		);

		assert_noop!(
			Crowdloan::complete_initialization(
				RuntimeOrigin::root(),
				init_block,
				init_block + VESTING * 2
			),
			Error::<Test>::RewardVecAlreadyInitialized,
		);
	});
}

#[test]
fn initialize_new_addresses_not_matching_funds() {
	empty().execute_with(|| {
		// The init relay block gets inserted
		roll_to(2);
		// Insert contributors
		let pairs = get_ed25519_pairs(2);
		let init_block = 1u64;
		// Total supply is 2500.Lets ensure inserting 2495 is not working.
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				([1u8; 32].into(), Some(1), 500u32.into()),
				([2u8; 32].into(), Some(2), 500u32.into()),
				(pairs[0].public().into(), None, 500u32.into()),
				(pairs[1].public().into(), None, 995u32.into()),
			]
		));
		assert_noop!(
			Crowdloan::complete_initialization(
				RuntimeOrigin::root(),
				init_block,
				init_block + VESTING
			),
			Error::<Test>::RewardsDoNotMatchFund
		);
	});
}

#[test]
fn initialize_new_addresses_with_batch() {
	empty().execute_with(|| {
		// This time should succeed trully
		roll_to(10);
		let init_block = 1u64;
		assert_ok!(mock::RuntimeCall::Utility(UtilityCall::batch_all {
			calls: vec![
				mock::RuntimeCall::Crowdloan(crate::Call::initialize_reward_vec {
					rewards: vec![([4u8; 32].into(), Some(3), 1250)],
				}),
				mock::RuntimeCall::Crowdloan(crate::Call::initialize_reward_vec {
					rewards: vec![([5u8; 32].into(), Some(1), 1250)],
				})
			]
		})
		.dispatch(RuntimeOrigin::root()));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));
		assert_eq!(Crowdloan::total_contributors(), 2);
		// Verify that the second ending block provider had no effect
		assert_eq!(
			Crowdloan::crowdloan_period(0),
			Some((init_block, init_block + VESTING))
		);

		// Batch calls always succeed. We just need to check the inner event
		assert_ok!(mock::RuntimeCall::Utility(UtilityCall::batch {
			calls: vec![mock::RuntimeCall::Crowdloan(
				crate::Call::initialize_reward_vec {
					rewards: vec![([4u8; 32].into(), Some(3), 500)]
				}
			)]
		})
		.dispatch(RuntimeOrigin::root()));

		let expected = vec![
			pallet_utility::Event::ItemCompleted,
			pallet_utility::Event::ItemCompleted,
			pallet_utility::Event::BatchCompleted,
			pallet_utility::Event::BatchInterrupted {
				index: 0,
				error: DispatchError::Module(ModuleError {
					index: 2,
					error: [8, 0, 0, 0],
					message: None,
				}),
			},
		];
		assert_eq!(batch_events(), expected);
	});
}

#[test]
fn floating_point_arithmetic_works() {
	empty().execute_with(|| {
		// The init relay block gets inserted
		let init_block = 1u64;
		assert_ok!(mock::RuntimeCall::Utility(UtilityCall::batch_all {
			calls: vec![
				mock::RuntimeCall::Crowdloan(crate::Call::initialize_reward_vec {
					rewards: vec![([4u8; 32].into(), Some(1), 1190)]
				}),
				mock::RuntimeCall::Crowdloan(crate::Call::initialize_reward_vec {
					rewards: vec![([5u8; 32].into(), Some(2), 1185)]
				}),
				// We will work with this. This has 100/8=12.5 payable per block
				mock::RuntimeCall::Crowdloan(crate::Call::initialize_reward_vec {
					rewards: vec![([3u8; 32].into(), Some(3), 125)]
				})
			]
		})
		.dispatch(RuntimeOrigin::root()));
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));
		assert_eq!(Crowdloan::total_contributors(), 3);

		assert_eq!(Crowdloan::accounts_payable(&3).unwrap().claimed_reward, 0);

		assert_ok!(Crowdloan::claim(RuntimeOrigin::signed(3), None));

		let expected = vec![crate::Event::RewardsPaid(3, 125)];
		assert_eq!(events(), expected);
	});
}

#[test]
fn test_initialization_errors() {
	empty().execute_with(|| {
		// The init relay block gets inserted
		roll_to(2);
		let init_block = 1u64;

		let pot = Crowdloan::get_crowdloan_allocation(0);

		// Too many contributors
		assert_noop!(
			Crowdloan::initialize_reward_vec(
				RuntimeOrigin::root(),
				vec![
					([1u8; 32].into(), Some(1), 1),
					([2u8; 32].into(), Some(2), 1),
					([3u8; 32].into(), Some(3), 1),
					([4u8; 32].into(), Some(4), 1),
					([5u8; 32].into(), Some(5), 1),
					([6u8; 32].into(), Some(6), 1),
					([7u8; 32].into(), Some(7), 1),
					([8u8; 32].into(), Some(8), 1),
					([9u8; 32].into(), Some(9), 1)
				]
			),
			Error::<Test>::TooManyContributors
		);

		// Go beyond fund pot
		assert_noop!(
			Crowdloan::initialize_reward_vec(
				RuntimeOrigin::root(),
				vec![([1u8; 32].into(), Some(1), pot + 1)]
			),
			Error::<Test>::BatchBeyondFundPot
		);

		// Dont fill rewards
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![([1u8; 32].into(), Some(1), pot - 1)]
		));

		// Fill rewards
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![([2u8; 32].into(), Some(2), 1)]
		));

		// Insert a non-valid vesting period
		assert_noop!(
			Crowdloan::complete_initialization(RuntimeOrigin::root(), init_block, init_block),
			Error::<Test>::VestingPeriodNonValid
		);

		// Cannot claim if we dont complete initialization
		assert_noop!(
			Crowdloan::claim(RuntimeOrigin::signed(1), None),
			Error::<Test>::RewardVecNotFullyInitializedYet
		);
		// Complete
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));

		// Cannot initialize again
		assert_noop!(
			Crowdloan::complete_initialization(RuntimeOrigin::root(), init_block, init_block),
			Error::<Test>::RewardVecAlreadyInitialized
		);
	});
}

#[test]
fn test_relay_signatures_can_change_reward_addresses() {
	empty().execute_with(|| {
		// 5 relay keys
		let pairs = get_ed25519_pairs(5);

		// The init relay block gets inserted
		roll_to(2);
		let init_block = 1u64;

		// We will have all pointint to the same reward account
		assert_ok!(Crowdloan::initialize_reward_vec(
			RuntimeOrigin::root(),
			vec![
				(pairs[0].public().into(), Some(1), 500u32.into()),
				(pairs[1].public().into(), Some(1), 500u32.into()),
				(pairs[2].public().into(), Some(1), 500u32.into()),
				(pairs[3].public().into(), Some(1), 500u32.into()),
				(pairs[4].public().into(), Some(1), 500u32.into())
			],
		));

		// Complete
		assert_ok!(Crowdloan::complete_initialization(
			RuntimeOrigin::root(),
			init_block,
			init_block + VESTING
		));

		let reward_info = Crowdloan::accounts_payable(&1).unwrap();

		// We should have all of them as contributors
		for pair in pairs.clone() {
			assert!(reward_info
				.contributed_relay_addresses
				.contains(&pair.public().into()))
		}

		// Threshold is set to 50%, so we need at least 3 votes to pass
		// Let's make sure that we dont pass with 2
		let mut payload = WRAPPED_BYTES_PREFIX.to_vec();
		payload.append(&mut TestSigantureNetworkIdentifier::get().to_vec());
		payload.append(&mut 2u64.encode());
		payload.append(&mut 1u64.encode());
		payload.append(&mut WRAPPED_BYTES_POSTFIX.to_vec());

		let mut insufficient_proofs: Vec<([u8; 32], MultiSignature)> = vec![];
		for i in 0..2 {
			insufficient_proofs.push((pairs[i].public().into(), pairs[i].sign(&payload).into()));
		}

		// Not sufficient proofs presented
		assert_noop!(
			Crowdloan::change_association_with_relay_keys(
				RuntimeOrigin::signed(1),
				2,
				1,
				insufficient_proofs.clone()
			),
			Error::<Test>::InsufficientNumberOfValidProofs
		);

		// With three votes we should passs
		let mut sufficient_proofs = insufficient_proofs.clone();

		// We push one more
		sufficient_proofs.push((pairs[2].public().into(), pairs[2].sign(&payload).into()));

		// This time should pass
		assert_ok!(Crowdloan::change_association_with_relay_keys(
			RuntimeOrigin::signed(1),
			2,
			1,
			sufficient_proofs.clone()
		));

		// 1 should no longer be payable
		assert!(Crowdloan::accounts_payable(&1).is_none());

		// 2 should be now payable
		let reward_info_2 = Crowdloan::accounts_payable(&2).unwrap();

		// The reward info should be identical
		assert_eq!(reward_info, reward_info_2);
	});
}

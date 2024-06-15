use crate::canisters::mu_smart_contract;
use crate::canisters::mu_smart_contract::Error;
use crate::canisters::mu_smart_contract::GetDeveloperResult;
use crate::canisters::mu_smart_contract::RequestEscrowWithdrawResult;
use crate::canisters::mu_smart_contract::Result_;
use crate::setup::TestCase;

use crate::canisters::mu_smart_contract::AppDto;
use crate::canisters::mu_smart_contract::AppState;
use crate::canisters::mu_smart_contract::DeployAppRequest;
use crate::canisters::mu_smart_contract::GetAppResult;
use crate::canisters::mu_smart_contract::RemoveAppResult;
use ic_ledger_types::AccountIdentifier;
use ic_ledger_types::Tokens;
use ic_ledger_types::DEFAULT_FEE;
use ic_ledger_types::DEFAULT_SUBACCOUNT;
use pocket_ic::call_candid_as;
use pocket_ic::common::rest::RawEffectivePrincipal;
use serde_bytes::ByteBuf;

#[test]
fn test_can_deploy_canister() {
    TestCase::setup();
}

#[test]
fn test_can_register_developer() {
    let test_case = TestCase::setup_with_registered_developer1();

    // Second request will fail
    let result = call_candid_as::<_, (Result_,)>(
        &test_case.pic,
        test_case.mu_smart_contract,
        RawEffectivePrincipal::None,
        test_case.developer1,
        "register_developer",
        ((),),
    )
    .unwrap();

    assert_eq!(Result_::Err(Error::DeveloperAccountAlreadyExist), result.0);

    // We can get developer information
    let result = call_candid_as::<_, (GetDeveloperResult,)>(
        &test_case.pic,
        test_case.mu_smart_contract,
        RawEffectivePrincipal::None,
        test_case.developer1,
        "get_developer",
        ((),),
    )
    .unwrap();
    assert!(matches!(result.0, GetDeveloperResult::Ok(_)));
}

#[test]
fn test_developers_can_withdraw_from_their_escrow_account() {
    let test_case = TestCase::setup_with_registered_developer1();

    let developer1_account = AccountIdentifier::new(&test_case.developer1, &DEFAULT_SUBACCOUNT);
    let developer_initial_balance = test_case.ledger_balance_of(developer1_account);
    assert_eq!(Tokens::from_e8s(10000000000), developer_initial_balance);

    let developer_info = match call_candid_as::<_, (GetDeveloperResult,)>(
        &test_case.pic,
        test_case.mu_smart_contract,
        RawEffectivePrincipal::None,
        test_case.developer1,
        "get_developer",
        ((),),
    )
    .unwrap()
    {
        (GetDeveloperResult::Ok(i),) => i,
        (GetDeveloperResult::Err(e),) => panic!("canister call failed: {e:?}"),
    };

    // Deposit ICP tokens to developer escrow account
    let escrow_account = AccountIdentifier::from_slice(&developer_info.escrow_account).unwrap();
    assert_eq!(
        Tokens::from_e8s(0),
        test_case.ledger_balance_of(escrow_account)
    );

    test_case
        .ledger_transfer(
            test_case.developer1,
            None,
            escrow_account,
            Tokens::from_e8s(250_000),
        )
        .unwrap();

    assert_eq!(
        Tokens::from_e8s(250_000),
        test_case.ledger_balance_of(escrow_account)
    );
    assert_eq!(
        developer_initial_balance - Tokens::from_e8s(250_000) - DEFAULT_FEE,
        test_case.ledger_balance_of(developer1_account)
    );

    // Request withdraw from escrow account
    match call_candid_as::<_, (RequestEscrowWithdrawResult,)>(
        &test_case.pic,
        test_case.mu_smart_contract,
        RawEffectivePrincipal::None,
        test_case.developer1,
        "request_escrow_withdraw",
        (
            developer1_account,
            mu_smart_contract::Tokens {
                e8s: 250_000 - DEFAULT_FEE.e8s(),
            },
        ),
    )
    .unwrap()
    {
        (RequestEscrowWithdrawResult::Ok(i),) => i,
        (RequestEscrowWithdrawResult::Err(e),) => panic!("canister call failed: {e:?}"),
    };

    assert_eq!(
        Tokens::from_e8s(0),
        test_case.ledger_balance_of(escrow_account)
    );
    assert_eq!(
        developer_initial_balance - (DEFAULT_FEE + DEFAULT_FEE),
        test_case.ledger_balance_of(developer1_account)
    );
}

#[test]
fn test_can_manage_app() {
    let test_case = TestCase::setup_with_registered_developer1();
    let developer_info = match call_candid_as::<_, (GetDeveloperResult,)>(
        &test_case.pic,
        test_case.mu_smart_contract,
        RawEffectivePrincipal::None,
        test_case.developer1,
        "get_developer",
        ((),),
    )
    .unwrap()
    {
        (GetDeveloperResult::Ok(i),) => i,
        (GetDeveloperResult::Err(e),) => panic!("canister call failed: {e:?}"),
    };

    // Can not deploy if doesn't have enough balance in escrow account
    match call_candid_as::<_, (Result_,)>(
        &test_case.pic,
        test_case.mu_smart_contract,
        RawEffectivePrincipal::None,
        test_case.developer1,
        "deploy_app",
        (DeployAppRequest {
            name: String::from("TestApp"),
            app_data: ByteBuf::from(b"invalid code"),
        },),
    )
    .unwrap()
    {
        (Result_::Err(Error::InsufficientBalanceForDeploy { was, needed }),)
            if was.e8s == 0 && needed.e8s == 1_000_000_000 =>
        {
            ()
        }
        (Result_::Ok(_),) => {
            panic!("Invalid result, should fail with `InsufficientBalanceForDeploy`")
        }
        (Result_::Err(e),) => panic!("canister call failed: {e:?}"),
    };

    // Deposit ICP tokens to developer escrow account
    let escrow_account = AccountIdentifier::from_slice(&developer_info.escrow_account).unwrap();
    assert_eq!(
        Tokens::from_e8s(0),
        test_case.ledger_balance_of(escrow_account)
    );

    test_case
        .ledger_transfer(
            test_case.developer1,
            None,
            escrow_account,
            Tokens::from_e8s(1_000_000_000),
        )
        .unwrap();

    assert_eq!(
        Tokens::from_e8s(1_000_000_000),
        test_case.ledger_balance_of(escrow_account)
    );

    // After depositing some tokens, we can deploy again
    let app_id = match call_candid_as::<_, (Result_,)>(
        &test_case.pic,
        test_case.mu_smart_contract,
        RawEffectivePrincipal::None,
        test_case.developer1,
        "deploy_app",
        (DeployAppRequest {
            name: String::from("TestApp"),
            app_data: ByteBuf::from(b"invalid code"),
        },),
    )
    .unwrap()
    {
        (Result_::Ok(a),) => a,
        (Result_::Err(e),) => panic!("canister call failed: {e:?}"),
    };

    // We can get app information
    match call_candid_as::<_, (GetAppResult,)>(
        &test_case.pic,
        test_case.mu_smart_contract,
        RawEffectivePrincipal::None,
        test_case.developer1,
        "get_app",
        (app_id,),
    )
    .unwrap()
    {
        (GetAppResult::Ok(a),) => assert_eq!(
            Some(AppDto {
                id: app_id,
                usages: vec![],
                state: AppState::Active {
                    name: String::from("TestApp"),
                    revision: 1,
                },
            }),
            a
        ),
        (GetAppResult::Err(e),) => panic!("canister call failed: {e:?}"),
    };

    // We can remove app
    match call_candid_as::<_, (RemoveAppResult,)>(
        &test_case.pic,
        test_case.mu_smart_contract,
        RawEffectivePrincipal::None,
        test_case.developer1,
        "remove_app",
        (app_id,),
    )
    .unwrap()
    {
        (RemoveAppResult::Ok,) => (),
        (RemoveAppResult::Err(e),) => panic!("canister call failed: {e:?}"),
    };
}

// TODO: Add test for `Request cycles` functionality

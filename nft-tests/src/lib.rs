#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
        DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_ACCOUNT_PUBLIC_KEY,
        DEFAULT_GENESIS_CONFIG, DEFAULT_GENESIS_CONFIG_HASH, DEFAULT_PAYMENT,
        DEFAULT_RUN_GENESIS_REQUEST,
    };
    use casper_execution_engine::core::{
        engine_state::{
            run_genesis_request::RunGenesisRequest, Error, ExecuteRequest, GenesisAccount,
        },
        execution,
    };
    use casper_types::{
        account::AccountHash, bytesrepr::FromBytes, gens::colliding_key_arb, runtime_args,
        system::mint, ApiError, CLTyped, CLValue, ContractHash, Key, Motes, PublicKey, RuntimeArgs,
        SecretKey, U256, U512,
    };

    const COLLECTION_NAME: &str = "collection_name";
    const NFT_CONTRACT_WASM: &str = "nft-installer.wasm";
    const CONTRACT_NAME: &str = "nft_contract";

    const NFT_TEST_COLLECTION: &str = "nft_test";
    const NFT_TEST_SYMBOL: &str = "TEST";

    pub const ENTRY_POINT_INIT: &str = "init";
    pub const ENTRY_POINT_SET_VARIABLES: &str = "set_variables";
    pub const ENTRY_POINT_MINT: &str = "mint";
    pub const ENTRY_POINT_BURN: &str = "burn";
    pub const ENTRY_POINT_TRANSFER: &str = "transfer";
    pub const ENTRY_POINT_BALANCE_OF: &str = "balance_of";

    const ARG_COLLECTION_NAME: &str = "collection_name";
    const ARG_COLLECTION_SYMBOL: &str = "collection_symbol";
    const ARG_TOTAL_TOKEN_SUPPLY: &str = "total_token_supply";

    const ARG_TOKEN_OWNER: &str = "token_owner";
    const ARG_TOKEN_RECEIVER: &str = "token_receiver";
    const ARG_TOKEN_NAME: &str = "token_name";
    const ARG_TOKEN_META: &str = "token_meta";
    const ARG_TOKEN_ID: &str = "token_id";

    #[test]
    fn should_install_contract() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_request_builder =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_collection_name(NFT_TEST_COLLECTION.to_string())
                .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
                .with_total_token_supply(U256::from(1));
        builder
            .exec(install_request_builder.build())
            .expect_success()
            .commit();

        let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        let nft_contract_key = account
            .named_keys()
            .get(CONTRACT_NAME)
            .expect("must have key in named keys");

        let query_result: String = query_stored_value(
            &mut builder,
            *nft_contract_key,
            vec![ARG_COLLECTION_NAME.to_string()],
        );

        assert_eq!(
            query_result,
            NFT_TEST_COLLECTION.to_string(),
            "collection_name initialized at installation should exist"
        );

        let query_result: String = query_stored_value(
            &mut builder,
            *nft_contract_key,
            vec![ARG_COLLECTION_SYMBOL.to_string()],
        );

        assert_eq!(
            query_result,
            NFT_TEST_SYMBOL.to_string(),
            "collection_symbol initialized at installation should exist"
        );
    }

    #[test]
    fn should_default_allow_minting() {
        let mut builder = InMemoryWasmTestBuilder::default();

        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        let install_builder =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM);
        // The allow_minting arg is defaulted to true if user does not provide value.
        builder.exec(install_builder.build()).expect_success();
    }

    #[test]
    fn should_reject_invalid_typed_name() {
        let install_request_builder =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_invalid_collection_name(
                    CLValue::from_t::<U256>(U256::from(0)).expect("expected CLValue"),
                );

        assert_expected_invalid_installer_request(install_request_builder, 18);
    }

    #[test]
    fn should_reject_invalid_typed_symbol() {
        let install_request_builder =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_invalid_collection_symbol(
                    CLValue::from_t::<U256>(U256::from(0)).expect("expected CLValue"),
                );

        assert_expected_invalid_installer_request(install_request_builder, 24);
    }

    #[test]
    fn should_reject_invalid_typed_total_token_supply() {
        let install_request_builder =
            InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
                .with_invalid_total_token_supply(
                    CLValue::from_t::<String>("".to_string()).expect("expected CLValue"),
                );

        assert_expected_invalid_installer_request(install_request_builder, 26);
    }

    // #[test]
    // fn should_mint_nft() {
    //     let mut builder = InMemoryWasmTestBuilder::default();
    //     builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    //     // Install contract
    //     let install_request = ExecuteRequestBuilder::standard(
    //         *DEFAULT_ACCOUNT_ADDR,
    //         NFT_CONTRACT_WASM,
    //         runtime_args! {
    //             ARG_COLLECTION_NAME => "Hans".to_string()
    //         },
    //     )
    //     .build();
    //     builder.exec(install_request).expect_success().commit();

    //     let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    //     let set_variables_request = ExecuteRequestBuilder::contract_call_by_name(
    //         *DEFAULT_ACCOUNT_ADDR,
    //         CONTRACT_NAME,
    //         ENTRY_POINT_SET_VARIABLES,
    //         runtime_args! {
    //             ARG_COLLECTION_NAME => "Austin".to_string()
    //         },
    //     )
    //     .build();
    //     builder
    //         .exec(set_variables_request)
    //         .expect_success()
    //         .commit();

    //     let nft_contract_key = account
    //         .named_keys()
    //         .get(CONTRACT_NAME)
    //         .expect("must have key in named keys");

    //     let query_result = builder
    //         .query(None, *nft_contract_key, &[COLLECTION_NAME.to_string()])
    //         .expect("must have stored value")
    //         .as_cl_value()
    //         .cloned()
    //         .expect("must have cl value")
    //         .into_t::<String>()
    //         .expect("must get string value");

    //     assert_eq!(query_result, "Austin".to_string());

    //     let mint_request = ExecuteRequestBuilder::contract_call_by_name(
    //         *DEFAULT_ACCOUNT_ADDR,
    //         CONTRACT_NAME,
    //         ENTRY_POINT_MINT,
    //         runtime_args! {
    //             ARG_TOKEN_OWNER => DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
    //             ARG_TOKEN_NAME => "Austin".to_string(),
    //             ARG_TOKEN_META => "Austin".to_string(),
    //         },
    //     )
    //     .build();
    //     builder.exec(mint_request).expect_success().commit();

    //     //This one will fail because: thou shalt not return!
    //     let balance_of_request = ExecuteRequestBuilder::contract_call_by_name(
    //         *DEFAULT_ACCOUNT_ADDR,
    //         CONTRACT_NAME,
    //         ENTRY_POINT_BALANCE_OF,
    //         runtime_args! {
    //             ARG_TOKEN_OWNER => DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
    //         },
    //     )
    //     .build();
    //     builder.exec(balance_of_request).expect_success().commit();
    // }

    // #[test]
    // fn should_transfer_nft_to_existing_account() {
    //     let mut builder = InMemoryWasmTestBuilder::default();
    //     builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    //     // Install contract
    //     let install_request = ExecuteRequestBuilder::standard(
    //         *DEFAULT_ACCOUNT_ADDR,
    //         NFT_CONTRACT_WASM,
    //         runtime_args! {
    //             ARG_COLLECTION_NAME => "Hans".to_string()
    //         },
    //     )
    //     .build();
    //     builder.exec(install_request).expect_success().commit();

    //     //Create reciever account
    //     let receiver_secret_key =
    //         SecretKey::ed25519_from_bytes([7; 32]).expect("failed to create secret key");
    //     let receiver_public_key = PublicKey::from(&receiver_secret_key);

    //     let receiver_account_hash = receiver_public_key.to_account_hash();

    //     let fund_receiver_request = ExecuteRequestBuilder::transfer(
    //         *DEFAULT_ACCOUNT_ADDR,
    //         runtime_args! {
    //             mint::ARG_AMOUNT => U512::from(30_000_000_000_000_u64), //The actual amount being tranferred?
    //             mint::ARG_TARGET => receiver_public_key.clone(),//Recipient account
    //             mint::ARG_ID => <Option::<u64>>::None, //What is ARG_ID for?
    //         },
    //     )
    //     .build();

    //     builder
    //         .exec(fund_receiver_request)
    //         .expect_success()
    //         .commit();

    //     let _ = builder.get_expected_account(receiver_account_hash);

    //     let mint_request = ExecuteRequestBuilder::contract_call_by_name(
    //         *DEFAULT_ACCOUNT_ADDR,
    //         CONTRACT_NAME,
    //         ENTRY_POINT_MINT,
    //         runtime_args! {
    //             ARG_TOKEN_OWNER => DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
    //             ARG_TOKEN_NAME => "Austin".to_string(),
    //             ARG_TOKEN_META => "Austin".to_string(),
    //         },
    //     )
    //     .build();
    //     builder.exec(mint_request).expect_success().commit();

    //     let transfer_request = ExecuteRequestBuilder::contract_call_by_name(
    //         *DEFAULT_ACCOUNT_ADDR,
    //         CONTRACT_NAME,
    //         ENTRY_POINT_TRANSFER,
    //         runtime_args! {
    //             ARG_TOKEN_OWNER => DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
    //             ARG_TOKEN_RECEIVER => receiver_public_key,

    //         },
    //     );
    // }

    //     let token_owner: PublicKey = runtime::get_named_arg(ARG_TOKEN_OWNER);
    //     let token_receiver: PublicKey = runtime::get_named_arg(ARG_TOKEN_RECEIVER);
    //     let token_id: U256 = runtime::get_named_arg(ARG_TOKEN_ID);

    fn assert_expected_invalid_installer_request(
        install_request_builder: InstallerRequestBuilder,
        expected_error_code: u16,
    ) {
        let mut builder = InMemoryWasmTestBuilder::default();

        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
        builder
            .exec(install_request_builder.build())
            .expect_failure(); // Should test against expected error

        let error = builder.get_error().expect("should have an error");
        assert_expected_error(error, expected_error_code);
    }

    fn assert_expected_error(error: Error, error_code: u16) {
        let actual = format!("{:?}", error);
        let expected = format!(
            "{:?}",
            Error::Exec(execution::Error::Revert(ApiError::User(error_code)))
        );

        assert_eq!(actual, expected, "Error should match {}", error_code)
    }

    fn query_stored_value<T: CLTyped + FromBytes>(
        builder: &mut InMemoryWasmTestBuilder,
        nft_contract_key: Key,
        path: Vec<String>,
    ) -> T {
        builder
            .query(None, nft_contract_key, &path)
            .expect("must have stored value")
            .as_cl_value()
            .cloned()
            .expect("must have cl value")
            .into_t::<T>()
            .expect("must get value")
    }

    struct InstallerRequestBuilder {
        account_hash: AccountHash,
        session_file: String,
        collection_name: CLValue,
        collection_symbol: CLValue,
        total_token_supply: CLValue,
    }

    impl InstallerRequestBuilder {
        fn new(account_hash: AccountHash, session_file: &str) -> Self {
            Self::default()
                .with_account_hash(account_hash)
                .with_session_file(session_file.to_string())
        }

        fn default() -> Self {
            InstallerRequestBuilder {
                account_hash: AccountHash::default(),
                session_file: String::default(),
                collection_name: CLValue::from_t("name".to_string())
                    .expect("name is legit CLValue"),
                collection_symbol: CLValue::from_t("SYM")
                    .expect("collection_symbol is legit CLValue"),
                total_token_supply: CLValue::from_t(U256::zero())
                    .expect("total_token_supply is legit CLValue"),
            }
        }

        fn with_account_hash(mut self, account_hash: AccountHash) -> Self {
            self.account_hash = account_hash;
            self
        }

        fn with_session_file(mut self, session_file: String) -> Self {
            self.session_file = session_file;
            self
        }

        fn with_collection_name(mut self, collection_name: String) -> Self {
            self.collection_name =
                CLValue::from_t(collection_name).expect("collection_name is legit CLValue");
            self
        }

        fn with_invalid_collection_name(mut self, collection_name: CLValue) -> Self {
            self.collection_name = collection_name;
            self
        }

        fn with_collection_symbol(mut self, collection_symbol: String) -> Self {
            self.collection_symbol =
                CLValue::from_t(collection_symbol).expect("collection_symbol is legit CLValue");
            self
        }

        fn with_invalid_collection_symbol(mut self, collection_symbol: CLValue) -> Self {
            self.collection_symbol = collection_symbol;
            self
        }

        fn with_total_token_supply(mut self, total_token_supply: U256) -> Self {
            self.total_token_supply =
                CLValue::from_t(total_token_supply).expect("total_token_supply is legit CLValue");
            self
        }

        fn with_invalid_total_token_supply(mut self, total_token_supply: CLValue) -> Self {
            self.total_token_supply = total_token_supply;
            self
        }

        fn build(self) -> ExecuteRequest {
            let mut runtime_args = RuntimeArgs::new();
            runtime_args.insert_cl_value(ARG_COLLECTION_NAME, self.collection_name);
            runtime_args.insert_cl_value(ARG_COLLECTION_SYMBOL, self.collection_symbol);
            runtime_args.insert_cl_value(ARG_TOTAL_TOKEN_SUPPLY, self.total_token_supply);
            ExecuteRequestBuilder::standard(self.account_hash, &self.session_file, runtime_args)
                .build()
        }
    }
}
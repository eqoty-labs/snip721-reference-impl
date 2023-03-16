#[cfg(test)]
mod tests {
    use std::any::Any;

    use cosmwasm_std::testing::*;
    use cosmwasm_std::{
        from_binary, to_binary, Addr, Api, Binary, Coin, OwnedDeps, Response, StdError, StdResult,
        SubMsg, Uint128, WasmMsg,
    };
    use cosmwasm_storage::ReadonlyPrefixedStorage;

    use crate::contract::{execute, instantiate, query};
    use crate::expiration::Expiration;
    use crate::inventory::Inventory;
    use crate::msg::{
        Burn, ContractStatus, ExecuteMsg, InstantiateConfig, InstantiateMsg, Mint,
        PostInstantiateCallback, QueryAnswer, QueryMsg, Send, Transfer,
    };
    use crate::royalties::{DisplayRoyalty, DisplayRoyaltyInfo, Royalty, RoyaltyInfo};
    use crate::state::{
        json_may_load, load, may_load, Config, CONFIG_KEY, PREFIX_INFOS, PREFIX_MAP_TO_ID,
        PREFIX_MAP_TO_INDEX,
    };
    use crate::token::{Extension, Metadata, Token};

    // Helper functions

    fn init_helper_default() -> (
        StdResult<Response>,
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("instantiator", &[]);
        let init_msg = InstantiateMsg {
            name: "sec721".to_string(),
            symbol: "S721".to_string(),
            admin: Some("admin".to_string()),
            entropy: "We're going to need a bigger boat".to_string(),
            royalty_info: None,
            config: None,
            post_init_callback: None,
            post_init_data: None,
        };

        (instantiate(&mut deps.as_mut(), &env, info, init_msg), deps)
    }

    fn init_helper_with_config(
        public_token_supply: bool,
        public_owner: bool,
        enable_sealed_metadata: bool,
        unwrapped_metadata_is_private: bool,
        minter_may_update_metadata: bool,
        owner_may_update_metadata: bool,
        enable_burn: bool,
    ) -> (
        StdResult<Response>,
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
    ) {
        let mut deps = mock_dependencies();

        let env = mock_env();
        let init_config: InstantiateConfig = from_binary(&Binary::from(
            format!(
                "{{\"public_token_supply\":{},
            \"public_owner\":{},
            \"enable_sealed_metadata\":{},
            \"unwrapped_metadata_is_private\":{},
            \"minter_may_update_metadata\":{},
            \"owner_may_update_metadata\":{},
            \"enable_burn\":{}}}",
                public_token_supply,
                public_owner,
                enable_sealed_metadata,
                unwrapped_metadata_is_private,
                minter_may_update_metadata,
                owner_may_update_metadata,
                enable_burn,
            )
            .as_bytes(),
        ))
        .unwrap();
        let info = mock_info("instantiator", &[]);
        let init_msg = InstantiateMsg {
            name: "sec721".to_string(),
            symbol: "S721".to_string(),
            admin: Some("admin".to_string()),
            entropy: "We're going to need a bigger boat".to_string(),
            royalty_info: None,
            config: Some(init_config),
            post_init_callback: None,
            post_init_data: None,
        };

        (instantiate(&mut deps.as_mut(), &env, info, init_msg), deps)
    }

    #[allow(clippy::too_many_arguments)]
    fn init_helper_royalties_with_config(
        royalty_info: Option<RoyaltyInfo>,
        public_token_supply: bool,
        public_owner: bool,
        enable_sealed_metadata: bool,
        unwrapped_metadata_is_private: bool,
        minter_may_update_metadata: bool,
        owner_may_update_metadata: bool,
        enable_burn: bool,
    ) -> (
        StdResult<Response>,
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
    ) {
        let mut deps = mock_dependencies();

        let env = mock_env();
        let init_config: InstantiateConfig = from_binary(&Binary::from(
            format!(
                "{{\"public_token_supply\":{},
            \"public_owner\":{},
            \"enable_sealed_metadata\":{},
            \"unwrapped_metadata_is_private\":{},
            \"minter_may_update_metadata\":{},
            \"owner_may_update_metadata\":{},
            \"enable_burn\":{}}}",
                public_token_supply,
                public_owner,
                enable_sealed_metadata,
                unwrapped_metadata_is_private,
                minter_may_update_metadata,
                owner_may_update_metadata,
                enable_burn,
            )
            .as_bytes(),
        ))
        .unwrap();
        let info = mock_info("instantiator", &[]);
        let init_msg = InstantiateMsg {
            name: "sec721".to_string(),
            symbol: "S721".to_string(),
            admin: Some("admin".to_string()),
            entropy: "We're going to need a bigger boat".to_string(),
            royalty_info,
            config: Some(init_config),
            post_init_callback: None,
            post_init_data: None,
        };

        (instantiate(&mut deps.as_mut(), &env, info, init_msg), deps)
    }

    fn extract_error_msg<T: Any>(error: StdResult<T>) -> String {
        match error {
            Ok(_response) => panic!("Expected error, but had Ok response"),
            Err(err) => match err {
                StdError::GenericErr { msg, .. } => msg,
                _ => panic!("Unexpected error result {:?}", err),
            },
        }
    }

    // Init tests

    #[test]
    fn test_init_sanity() {
        // test default
        let (init_result, deps) = init_helper_default();
        assert_eq!(init_result.unwrap(), Response::default());
        let config: Config = load(&deps.storage, CONFIG_KEY).unwrap();
        assert_eq!(config.status, ContractStatus::Normal.to_u8());
        assert_eq!(config.mint_cnt, 0);
        assert_eq!(config.tx_cnt, 0);
        assert_eq!(config.name, "sec721".to_string());
        assert_eq!(config.admin, deps.api.addr_canonicalize("admin").unwrap());
        assert_eq!(config.symbol, "S721".to_string());
        assert!(!config.token_supply_is_public);
        assert!(!config.owner_is_public);
        assert!(!config.sealed_metadata_is_enabled);
        assert!(!config.unwrap_to_private);
        assert!(config.minter_may_update_metadata);
        assert!(!config.owner_may_update_metadata);
        assert!(!config.burn_is_enabled);

        // test config specification
        let (init_result, deps) =
            init_helper_with_config(true, true, true, true, false, true, false);
        assert_eq!(init_result.unwrap(), Response::default());
        let config: Config = load(&deps.storage, CONFIG_KEY).unwrap();
        assert_eq!(config.status, ContractStatus::Normal.to_u8());
        assert_eq!(config.mint_cnt, 0);
        assert_eq!(config.tx_cnt, 0);
        assert_eq!(config.name, "sec721".to_string());
        assert_eq!(config.admin, deps.api.addr_canonicalize("admin").unwrap());
        assert_eq!(config.symbol, "S721".to_string());
        assert!(config.token_supply_is_public);
        assert!(config.owner_is_public);
        assert!(config.sealed_metadata_is_enabled);
        assert!(config.unwrap_to_private);
        assert!(!config.minter_may_update_metadata);
        assert!(config.owner_may_update_metadata);
        assert!(!config.burn_is_enabled);

        // test post init callback
        let mut deps = mock_dependencies();
        let env = mock_env();
        // just picking a random short HandleMsg that wouldn't really make sense
        let post_init_msg = to_binary(&ExecuteMsg::MakeOwnershipPrivate { padding: None }).unwrap();
        let post_init_send = vec![Coin {
            amount: Uint128::new(100),
            denom: "uscrt".to_string(),
        }];
        let post_init_callback = Some(PostInstantiateCallback {
            msg: post_init_msg.clone(),
            contract_address: "spawner".to_string(),
            code_hash: "spawner hash".to_string(),
            send: post_init_send.clone(),
        });
        let info = mock_info("instantiator", &[]);
        let init_msg = InstantiateMsg {
            name: "sec721".to_string(),
            symbol: "S721".to_string(),
            admin: Some("admin".to_string()),
            entropy: "We're going to need a bigger boat".to_string(),
            royalty_info: None,
            config: None,
            post_init_callback,
            post_init_data: None,
        };

        let init_response = instantiate(&mut deps.as_mut(), &env, info, init_msg).unwrap();
        assert_eq!(
            init_response.messages,
            vec![SubMsg::new(WasmMsg::Execute {
                msg: post_init_msg,
                contract_addr: "spawner".to_string(),
                code_hash: "spawner hash".to_string(),
                funds: post_init_send,
            })]
        );

        // test config specification with default royalties
        let royalties = RoyaltyInfo {
            decimal_places_in_rates: 2,
            royalties: vec![
                Royalty {
                    recipient: "alice".to_string(),
                    rate: 10,
                },
                Royalty {
                    recipient: "bob".to_string(),
                    rate: 5,
                },
            ],
        };

        let expected_hidden = DisplayRoyaltyInfo {
            decimal_places_in_rates: 2,
            royalties: vec![
                DisplayRoyalty {
                    recipient: None,
                    rate: 10,
                },
                DisplayRoyalty {
                    recipient: None,
                    rate: 5,
                },
            ],
        };

        let (init_result, deps) = init_helper_royalties_with_config(
            Some(royalties),
            true,
            true,
            true,
            true,
            false,
            true,
            false,
        );
        assert_eq!(init_result.unwrap(), Response::default());
        let config: Config = load(&deps.storage, CONFIG_KEY).unwrap();
        assert_eq!(config.status, ContractStatus::Normal.to_u8());
        assert_eq!(config.mint_cnt, 0);
        assert_eq!(config.tx_cnt, 0);
        assert_eq!(config.name, "sec721".to_string());
        assert_eq!(config.admin, deps.api.addr_canonicalize("admin").unwrap());
        assert_eq!(config.symbol, "S721".to_string());
        assert!(config.token_supply_is_public);
        assert!(config.owner_is_public);
        assert!(config.sealed_metadata_is_enabled);
        assert!(config.unwrap_to_private);
        assert!(!config.minter_may_update_metadata);
        assert!(config.owner_may_update_metadata);
        assert!(!config.burn_is_enabled);

        let query_msg = QueryMsg::RoyaltyInfo {
            token_id: None,
            viewer: None,
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::RoyaltyInfo { royalty_info } => {
                assert_eq!(royalty_info, Some(expected_hidden));
            }
            _ => panic!("unexpected"),
        }
    }

    // Handle tests

    // test no royalties if non-transferable
    #[test]
    fn test_no_royalties_if_non_transferable() {
        let royalties = RoyaltyInfo {
            decimal_places_in_rates: 2,
            royalties: vec![
                Royalty {
                    recipient: "alice".to_string(),
                    rate: 10,
                },
                Royalty {
                    recipient: "bob".to_string(),
                    rate: 5,
                },
            ],
        };

        let (init_result, mut deps) = init_helper_royalties_with_config(
            Some(royalties.clone()),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
        assert_eq!(init_result.unwrap(), Response::default());

        let mints = vec![
            Mint {
                token_id: Some("TrySetRoys".to_string()),
                owner: None,
                public_metadata: None,
                private_metadata: None,
                royalty_info: Some(royalties.clone()),
                transferable: Some(false),
                serial_number: None,
                memo: None,
            },
            Mint {
                token_id: Some("TryDefaultRoys".to_string()),
                owner: None,
                public_metadata: None,
                private_metadata: None,
                royalty_info: None,
                transferable: Some(false),
                serial_number: None,
                memo: None,
            },
        ];

        let execute_msg = ExecuteMsg::BatchMintNft {
            mints,
            padding: None,
        };
        let handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            execute_msg,
        );
        assert!(handle_result.is_ok());

        // verify there are no royalties when trying to specify on mint
        let query_msg = QueryMsg::RoyaltyInfo {
            token_id: Some("TrySetRoys".to_string()),
            viewer: None,
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::RoyaltyInfo { royalty_info } => {
                assert!(royalty_info.is_none());
            }
            _ => panic!("unexpected"),
        }

        // verify there are no royalties coming from the default
        let query_msg = QueryMsg::RoyaltyInfo {
            token_id: Some("TryDefaultRoys".to_string()),
            viewer: None,
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::RoyaltyInfo { royalty_info } => {
                assert!(royalty_info.is_none());
            }
            _ => panic!("unexpected"),
        }

        // test trying SetRoyaltyInfo on a non-transferable token
        let execute_msg = ExecuteMsg::SetRoyaltyInfo {
            token_id: Some("TryDefaultRoys".to_string()),
            royalty_info: Some(royalties),
            padding: None,
        };
        let handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            execute_msg,
        );
        let error = extract_error_msg(handle_result);
        assert!(
            error.contains("Non-transferable tokens can not be sold, so royalties are meaningless")
        );
    }

    // test trying to transfer/send non-transferable
    #[test]
    fn test_xfer_send_non_transferable() {
        let (init_result, mut deps) =
            init_helper_with_config(false, false, false, false, false, false, false);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let alice = "alice".to_string();
        let bob = "bob".to_string();

        let mints = vec![
            Mint {
                token_id: Some("NFT1".to_string()),
                owner: Some(alice.clone()),
                public_metadata: None,
                private_metadata: None,
                royalty_info: None,
                transferable: Some(false),
                serial_number: None,
                memo: None,
            },
            Mint {
                token_id: Some("NFT2".to_string()),
                owner: Some(alice),
                public_metadata: None,
                private_metadata: None,
                royalty_info: None,
                transferable: Some(false),
                serial_number: None,
                memo: None,
            },
        ];

        let execute_msg = ExecuteMsg::BatchMintNft {
            mints,
            padding: None,
        };
        let handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            execute_msg,
        );
        assert!(handle_result.is_ok());

        // verify TransferNft fails on a non-transferable token
        let execute_msg = ExecuteMsg::TransferNft {
            recipient: bob.clone(),
            token_id: "NFT1".to_string(),
            memo: None,
            padding: None,
        };
        let handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            execute_msg,
        );
        let error = extract_error_msg(handle_result);
        assert!(error.contains("Token ID: NFT1 is non-transferable"));

        // verify BatchTransferNft fails on a non-transferable token
        let transfers = vec![
            Transfer {
                recipient: bob.clone(),
                token_ids: vec!["NFT2".to_string()],
                memo: None,
            },
            Transfer {
                recipient: bob.clone(),
                token_ids: vec!["NFT1".to_string()],
                memo: None,
            },
        ];
        let execute_msg = ExecuteMsg::BatchTransferNft {
            transfers,
            padding: None,
        };
        let handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            execute_msg,
        );
        let error = extract_error_msg(handle_result);
        assert!(error.contains("Token ID: NFT2 is non-transferable"));

        // verify SendNft fails on a non-transferable token
        let execute_msg = ExecuteMsg::SendNft {
            contract: bob.clone(),
            receiver_info: None,
            token_id: "NFT1".to_string(),
            msg: None,
            memo: None,
            padding: None,
        };
        let handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            execute_msg,
        );
        let error = extract_error_msg(handle_result);
        assert!(error.contains("Token ID: NFT1 is non-transferable"));

        // verify BatchSendNft fails on a non-transferable token
        let sends = vec![
            Send {
                contract: bob.clone(),
                receiver_info: None,
                token_ids: vec!["NFT2".to_string()],
                msg: None,
                memo: None,
            },
            Send {
                contract: bob,
                receiver_info: None,
                token_ids: vec!["NFT1".to_string()],
                msg: None,
                memo: None,
            },
        ];
        let execute_msg = ExecuteMsg::BatchSendNft {
            sends,
            padding: None,
        };
        let handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            execute_msg,
        );
        let error = extract_error_msg(handle_result);
        assert!(error.contains("Token ID: NFT2 is non-transferable"));
    }

    // test non-transferable is always burnable
    #[test]
    fn test_burn_non_transferable() {
        let (init_result, mut deps) =
            init_helper_with_config(false, false, false, false, false, false, false);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let tok_key = 0u32.to_le_bytes();
        let tok2_key = 1u32.to_le_bytes();
        let tok3_key = 2u32.to_le_bytes();
        let alice = "alice".to_string();
        let alice_raw = deps.api.addr_canonicalize(&alice).unwrap();

        let mints = vec![
            Mint {
                token_id: Some("NFT1".to_string()),
                owner: Some(alice.clone()),
                public_metadata: None,
                private_metadata: None,
                royalty_info: None,
                transferable: Some(false),
                serial_number: None,
                memo: None,
            },
            Mint {
                token_id: Some("NFT2".to_string()),
                owner: Some(alice.clone()),
                public_metadata: None,
                private_metadata: None,
                royalty_info: None,
                transferable: Some(false),
                serial_number: None,
                memo: None,
            },
            Mint {
                token_id: Some("NFT3".to_string()),
                owner: Some(alice),
                public_metadata: None,
                private_metadata: None,
                royalty_info: None,
                transferable: Some(false),
                serial_number: None,
                memo: None,
            },
        ];

        let execute_msg = ExecuteMsg::BatchMintNft {
            mints,
            padding: None,
        };
        let handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            execute_msg,
        );
        assert!(handle_result.is_ok());

        // verify BurnNft works on a non-transferable token even when burn is disabled
        let execute_msg = ExecuteMsg::BurnNft {
            token_id: "NFT1".to_string(),
            memo: None,
            padding: None,
        };
        let handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            execute_msg,
        );
        assert!(handle_result.is_ok());
        // confirm token was removed from the maps
        let map2idx = ReadonlyPrefixedStorage::new(&deps.storage, PREFIX_MAP_TO_INDEX);
        let index: Option<u32> = may_load(&map2idx, "NFT1".as_bytes()).unwrap();
        assert!(index.is_none());
        let map2id = ReadonlyPrefixedStorage::new(&deps.storage, PREFIX_MAP_TO_ID);
        let id: Option<String> = may_load(&map2id, &tok_key).unwrap();
        assert!(id.is_none());
        // confirm token info was deleted from storage
        let info_store = ReadonlyPrefixedStorage::new(&deps.storage, PREFIX_INFOS);
        let token: Option<Token> = json_may_load(&info_store, &tok_key).unwrap();
        assert!(token.is_none());
        // confirm the token was removed from the owner's list
        let inventory = Inventory::new(&deps.storage, alice_raw.clone()).unwrap();
        assert_eq!(inventory.cnt, 2);
        assert!(!inventory.contains(&deps.storage, 0).unwrap());
        assert!(inventory.contains(&deps.storage, 1).unwrap());
        assert!(inventory.contains(&deps.storage, 2).unwrap());

        // verify BatchBurnNft works on non-transferable tokens even when burn is disabled
        let burns = vec![Burn {
            token_ids: vec!["NFT2".to_string(), "NFT3".to_string()],
            memo: None,
        }];
        let execute_msg = ExecuteMsg::BatchBurnNft {
            burns,
            padding: None,
        };
        let handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            execute_msg,
        );
        assert!(handle_result.is_ok());
        // confirm tokens were removed from the maps
        let map2idx = ReadonlyPrefixedStorage::new(&deps.storage, PREFIX_MAP_TO_INDEX);
        let index: Option<u32> = may_load(&map2idx, "NFT2".as_bytes()).unwrap();
        assert!(index.is_none());
        let index: Option<u32> = may_load(&map2idx, "NFT3".as_bytes()).unwrap();
        assert!(index.is_none());
        let map2id = ReadonlyPrefixedStorage::new(&deps.storage, PREFIX_MAP_TO_ID);
        let id: Option<String> = may_load(&map2id, &tok2_key).unwrap();
        assert!(id.is_none());
        let id: Option<String> = may_load(&map2id, &tok3_key).unwrap();
        assert!(id.is_none());
        // confirm token infos were deleted from storage
        let info_store = ReadonlyPrefixedStorage::new(&deps.storage, PREFIX_INFOS);
        let token: Option<Token> = json_may_load(&info_store, &tok2_key).unwrap();
        assert!(token.is_none());
        let token: Option<Token> = json_may_load(&info_store, &tok3_key).unwrap();
        assert!(token.is_none());
        // confirm the tokens were removed from the owner's list
        let inventory = Inventory::new(&deps.storage, alice_raw).unwrap();
        assert_eq!(inventory.cnt, 0);
        assert!(!inventory.contains(&deps.storage, 1).unwrap());
        assert!(!inventory.contains(&deps.storage, 2).unwrap());
    }

    // query tests

    // test new transferable and unwrapped fields of NftDossier query
    #[test]
    fn test_query_nft_dossier() {
        let (init_result, mut deps) =
            init_helper_with_config(false, true, true, false, true, false, true);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let public_meta = Metadata {
            token_uri: None,
            extension: Some(Extension {
                name: Some("Name1".to_string()),
                description: Some("PubDesc1".to_string()),
                image: Some("PubUri1".to_string()),
                ..Extension::default()
            }),
        };
        let private_meta = Metadata {
            token_uri: None,
            extension: Some(Extension {
                name: Some("PrivName1".to_string()),
                description: Some("PrivDesc1".to_string()),
                image: Some("PrivUri1".to_string()),
                ..Extension::default()
            }),
        };
        let alice = "alice".to_string();

        let execute_msg = ExecuteMsg::MintNft {
            token_id: Some("NFT1".to_string()),
            owner: Some(alice.clone()),
            public_metadata: Some(public_meta.clone()),
            private_metadata: Some(private_meta),
            royalty_info: None,
            serial_number: None,
            transferable: Some(false),
            memo: None,
            padding: None,
        };
        let handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            execute_msg,
        );
        assert!(handle_result.is_ok());

        // test viewer not given, contract has public ownership
        let query_msg = QueryMsg::NftDossier {
            token_id: "NFT1".to_string(),
            viewer: None,
            include_expired: None,
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg);
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::NftDossier {
                owner,
                public_metadata,
                private_metadata,
                royalty_info: _,
                mint_run_info: _,
                transferable,
                unwrapped,
                display_private_metadata_error,
                owner_is_public,
                public_ownership_expiration,
                private_metadata_is_public,
                private_metadata_is_public_expiration,
                token_approvals,
                inventory_approvals,
            } => {
                assert_eq!(owner, Some(Addr::unchecked(alice)));
                assert_eq!(public_metadata, Some(public_meta));
                assert!(private_metadata.is_none());
                assert_eq!(
                    display_private_metadata_error,
                    Some("You are not authorized to perform this action on token NFT1".to_string())
                );
                assert!(owner_is_public);
                assert!(!transferable);
                assert!(!unwrapped);
                assert_eq!(public_ownership_expiration, Some(Expiration::Never));
                assert!(!private_metadata_is_public);
                assert!(private_metadata_is_public_expiration.is_none());
                assert!(token_approvals.is_none());
                assert!(inventory_approvals.is_none());
            }
            _ => panic!("unexpected"),
        }
    }

    // test IsTransferable query
    #[test]
    fn test_is_transferable() {
        let (init_result, deps) =
            init_helper_with_config(true, true, false, false, true, false, true);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        // test token not found when supply is public
        let query_msg = QueryMsg::IsTransferable {
            token_id: "NFT1".to_string(),
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg);
        let error = extract_error_msg(query_result);
        assert!(error.contains("Token ID: NFT1 not found"));

        let (init_result, deps) =
            init_helper_with_config(false, true, false, false, true, false, true);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        // test token not found when supply is private
        let query_msg = QueryMsg::IsTransferable {
            token_id: "NFT1".to_string(),
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg);
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::IsTransferable {
                token_is_transferable,
            } => {
                assert!(token_is_transferable);
            }
            _ => panic!("unexpected"),
        }

        let (init_result, mut deps) =
            init_helper_with_config(false, true, true, false, true, false, true);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let mints = vec![
            Mint {
                token_id: Some("NFT1".to_string()),
                owner: None,
                public_metadata: None,
                private_metadata: None,
                royalty_info: None,
                transferable: Some(false),
                serial_number: None,
                memo: None,
            },
            Mint {
                token_id: Some("NFT2".to_string()),
                owner: None,
                public_metadata: None,
                private_metadata: None,
                royalty_info: None,
                transferable: None,
                serial_number: None,
                memo: None,
            },
        ];

        let execute_msg = ExecuteMsg::BatchMintNft {
            mints,
            padding: None,
        };
        let handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            execute_msg,
        );
        assert!(handle_result.is_ok());

        // test IsTransferable on a non-transferable token
        let query_msg = QueryMsg::IsTransferable {
            token_id: "NFT1".to_string(),
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg);
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::IsTransferable {
                token_is_transferable,
            } => {
                assert!(!token_is_transferable);
            }
            _ => panic!("unexpected"),
        }

        // test IsTransferable on a transferable token
        let query_msg = QueryMsg::IsTransferable {
            token_id: "NFT2".to_string(),
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg);
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::IsTransferable {
                token_is_transferable,
            } => {
                assert!(token_is_transferable);
            }
            _ => panic!("unexpected"),
        }
    }

    // test VerifyTransferApproval query on non-transferable tokens
    #[test]
    fn test_verify_transfer_approval() {
        let (init_result, mut deps) =
            init_helper_with_config(false, false, false, false, true, false, true);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let alice = "alice".to_string();
        let bob = "bob".to_string();

        let execute_msg = ExecuteMsg::SetViewingKey {
            key: "akey".to_string(),
            padding: None,
        };
        let handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            execute_msg,
        );
        assert!(handle_result.is_ok());

        let execute_msg = ExecuteMsg::SetViewingKey {
            key: "bkey".to_string(),
            padding: None,
        };
        let handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            execute_msg,
        );
        assert!(handle_result.is_ok());

        let execute_msg = ExecuteMsg::MintNft {
            token_id: Some("NFT1".to_string()),
            owner: Some(alice.clone()),
            public_metadata: None,
            private_metadata: None,
            royalty_info: None,
            serial_number: None,
            transferable: Some(false),
            memo: None,
            padding: None,
        };
        let handle_result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            execute_msg,
        );
        assert!(handle_result.is_ok());

        // verify that alice does not have transfer approval despite owning the non-transferable token
        let query_msg = QueryMsg::VerifyTransferApproval {
            token_ids: vec!["NFT1".to_string()],
            address: alice,
            viewing_key: "akey".to_string(),
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg);
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::VerifyTransferApproval {
                approved_for_all,
                first_unapproved_token,
            } => {
                assert!(!approved_for_all);
                assert_eq!(first_unapproved_token.unwrap(), "NFT1".to_string());
            }
            _ => panic!("unexpected"),
        }

        // also verify that bob does not have transfer approval
        let query_msg = QueryMsg::VerifyTransferApproval {
            token_ids: vec!["NFT1".to_string()],
            address: bob,
            viewing_key: "bkey".to_string(),
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg);
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::VerifyTransferApproval {
                approved_for_all,
                first_unapproved_token,
            } => {
                assert!(!approved_for_all);
                assert_eq!(first_unapproved_token, Some("NFT1".to_string()));
            }
            _ => panic!("unexpected"),
        }
    }
}

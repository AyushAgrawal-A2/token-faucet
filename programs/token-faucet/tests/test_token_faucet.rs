use {
    anchor_lang::{
        solana_program::{instruction::Instruction, program_option::COption, program_pack::Pack},
        system_program, AccountDeserialize, InstructionData, ToAccountMetas,
    },
    anchor_spl::{
        associated_token,
        token::{self, spl_token::state::Mint, TokenAccount},
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Address, Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    token_faucet::{FaucetConfig, MintTimeout, FAUCET_SEED, MINT_SEED, MINT_TIMEOUT_SEED},
};

const ADMIN_BYTES: [u8; 32] = [
    141, 230, 113, 141, 215, 26, 159, 237, 198, 141, 191, 180, 113, 69, 172, 108, 74, 130, 74, 247,
    242, 0, 79, 139, 170, 129, 229, 67, 68, 124, 32, 30,
];

#[test]
fn test_token_faucet() {
    let program_id = token_faucet::id();
    let admin = Keypair::new_from_array(ADMIN_BYTES);
    let payer = Keypair::new();
    let payer_address = payer.pubkey();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/token_faucet.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&admin.pubkey(), 1_000_000_000).unwrap();
    svm.airdrop(&payer_address, 1_000_000_000).unwrap();

    let decimals = 6;
    let max_supply = 100_000_000;
    let mint_timeout = 60 * 60 * 1_000;
    let mint_limit = 10_000_000;

    let seed = 1u64;
    let seed_bytes = seed.to_le_bytes();

    let faucet_config_seeds = [FAUCET_SEED.as_bytes(), seed_bytes.as_ref()];
    let (faucet_config_pda, faucet_config_bump) =
        Address::derive_program_address(&faucet_config_seeds, &program_id).unwrap();

    let mint_seeds = [MINT_SEED.as_bytes(), seed_bytes.as_ref()];
    let (mint_pda, mint_bump) = Address::derive_program_address(&mint_seeds, &program_id).unwrap();

    let mint_timeout_seeds = [
        MINT_TIMEOUT_SEED.as_bytes(),
        seed_bytes.as_ref(),
        payer_address.as_ref(),
    ];
    let (mint_timeout_pda, mint_timeout_bump) =
        Address::derive_program_address(&mint_timeout_seeds, &program_id).unwrap();

    let admin_token_ata =
        associated_token::get_associated_token_address(&admin.pubkey(), &mint_pda);
    let payer_token_ata = associated_token::get_associated_token_address(&payer_address, &mint_pda);

    let instruction = Instruction::new_with_bytes(
        program_id,
        &token_faucet::instruction::Initialize {
            _seed: seed,
            _decimals: decimals,
            max_supply: 0,
            mint_timeout: 0,
            mint_limit: 0,
        }
        .data(),
        token_faucet::accounts::Initialize {
            admin: admin.pubkey(),
            faucet_config: faucet_config_pda,
            mint: mint_pda,
            token_program: token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&admin.pubkey()), &blockhash);
    let tx =
        VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[admin.insecure_clone()])
            .unwrap();
    svm.send_transaction(tx).unwrap();
    let faucet_config_account = svm.get_account(&faucet_config_pda).unwrap();
    let faucet_config =
        FaucetConfig::try_deserialize(&mut faucet_config_account.data.as_slice()).unwrap();
    assert_eq!(faucet_config.max_supply, 0);
    assert_eq!(faucet_config.mint_timeout, 0);
    assert_eq!(faucet_config.mint_limit, 0);
    let mint_account = svm.get_account(&mint_pda).unwrap();
    let mint = Mint::unpack(mint_account.data.as_slice()).unwrap();
    assert_eq!(mint.decimals, decimals);
    assert_eq!(mint.mint_authority, COption::Some(mint_pda));

    let instruction = Instruction::new_with_bytes(
        program_id,
        &token_faucet::instruction::MintToken {
            seed,
            amount: 1_000_000,
        }
        .data(),
        token_faucet::accounts::MintToken {
            payer: payer_address,
            token_ata: payer_token_ata,
            faucet_config: faucet_config_pda,
            mint_timeout: mint_timeout_pda,
            mint: mint_pda,
            system_program: system_program::ID,
            token_program: token::ID,
            associated_token_program: associated_token::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer_address), &blockhash);
    let tx =
        VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer.insecure_clone()])
            .unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_err());

    let instruction = Instruction::new_with_bytes(
        program_id,
        &token_faucet::instruction::UpdateConfig {
            _seed: seed,
            max_supply,
            mint_timeout,
            mint_limit,
        }
        .data(),
        token_faucet::accounts::UpdateConfig {
            admin: admin.pubkey(),
            faucet_config: faucet_config_pda,
            mint: mint_pda,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&admin.pubkey()), &blockhash);
    let tx =
        VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[admin.insecure_clone()])
            .unwrap();
    svm.send_transaction(tx).unwrap();
    let faucet_config_account = svm.get_account(&faucet_config_pda).unwrap();
    let faucet_config =
        FaucetConfig::try_deserialize(&mut faucet_config_account.data.as_slice()).unwrap();
    assert_eq!(faucet_config.max_supply, max_supply);
    assert_eq!(faucet_config.mint_timeout, mint_timeout);
    assert_eq!(faucet_config.mint_limit, mint_limit);

    let instruction = Instruction::new_with_bytes(
        program_id,
        &token_faucet::instruction::MintToken {
            seed,
            amount: 5_000_000,
        }
        .data(),
        token_faucet::accounts::MintToken {
            payer: payer_address,
            token_ata: payer_token_ata,
            faucet_config: faucet_config_pda,
            mint_timeout: mint_timeout_pda,
            mint: mint_pda,
            system_program: system_program::ID,
            token_program: token::ID,
            associated_token_program: associated_token::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer_address), &blockhash);
    let tx =
        VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer.insecure_clone()])
            .unwrap();
    svm.send_transaction(tx).unwrap();
    let payer_token_ata_account = svm.get_account(&payer_token_ata).unwrap();
    let payer_token_ata_data =
        TokenAccount::try_deserialize(&mut payer_token_ata_account.data.as_slice()).unwrap();
    assert_eq!(payer_token_ata_data.amount, 5_000_000);
    let mint_timeout_pda_account = svm.get_account(&mint_timeout_pda).unwrap();
    let mint_timeout_pda_data =
        MintTimeout::try_deserialize(&mut mint_timeout_pda_account.data.as_slice()).unwrap();
    assert_eq!(mint_timeout_pda_data.amount_minted, 5_000_000);

    let instruction = Instruction::new_with_bytes(
        program_id,
        &token_faucet::instruction::TransferToken {
            _seed: seed,
            amount: 5_000_000,
        }
        .data(),
        token_faucet::accounts::TransferToken {
            sender: payer_address,
            sender_ata: payer_token_ata,
            recipient: admin.pubkey(),
            recipient_ata: admin_token_ata,
            mint: mint_pda,
            system_program: system_program::ID,
            token_program: token::ID,
            associated_token_program: associated_token::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer_address), &blockhash);
    let tx =
        VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer.insecure_clone()])
            .unwrap();
    svm.send_transaction(tx).unwrap();
    let payer_token_ata_account = svm.get_account(&payer_token_ata).unwrap();
    let payer_token_ata_data =
        TokenAccount::try_deserialize(&mut payer_token_ata_account.data.as_slice()).unwrap();
    assert_eq!(payer_token_ata_data.amount, 0);
    let admin_token_ata_account = svm.get_account(&admin_token_ata).unwrap();
    let admin_token_ata_data =
        TokenAccount::try_deserialize(&mut admin_token_ata_account.data.as_slice()).unwrap();
    assert_eq!(admin_token_ata_data.amount, 5_000_000);
}

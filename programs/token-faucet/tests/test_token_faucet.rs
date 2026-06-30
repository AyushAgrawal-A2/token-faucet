use {
    anchor_lang::{solana_program::instruction::Instruction, InstructionData, ToAccountMetas},
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
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
    println!("{:?}", payer.pubkey());
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/token_faucet.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    // let instruction = Instruction::new_with_bytes(
    //     program_id,
    //     &token_faucet::instruction::Initialize {}.data(),
    //     token_faucet::accounts::Initialize {}.to_account_metas(None),
    // );

    // let blockhash = svm.latest_blockhash();
    // let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    // let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();

    // let res = svm.send_transaction(tx);
    // assert!(res.is_ok());
}

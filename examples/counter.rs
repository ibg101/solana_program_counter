use solana_sdk::{
    hash::Hash, 
    instruction::{AccountMeta, Instruction}, 
    message::Message, 
    native_token::LAMPORTS_PER_SOL, 
    pubkey::Pubkey, 
    signature::{Keypair, Signature}, 
    signer::Signer, 
    system_program,
    transaction::Transaction
};
use solana_client::nonblocking::rpc_client::RpcClient;
// use counter::instruction::CounterInstruction;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    env_logger::init();

    let url: String = String::from("http://127.0.0.1:8899");
    let rpc_client: RpcClient = RpcClient::new(url);

    // 1. init payer (if you want to have the same payer every time this binary gets executed -> set `MEW_PAYER=false` in .env)
    let (payer, payer_pkey) = init_payer(&rpc_client).await?;

    // 2. derive PDA
    let (pda, _bump) = Pubkey::find_program_address(
        &[b"counter", payer_pkey.as_ref()], 
        &counter::ID
    );

    // 3. create initialize_counter instruction & transaction
    let initialize_ix: Instruction = Instruction::new_with_bytes(
        counter::ID, 
        &[0],  // 0 stays for instr_type, which is CounterInstruction::InitializeCounter
        vec![
            AccountMeta::new(payer_pkey, true),
            AccountMeta::new(pda, false),
            AccountMeta::new_readonly(system_program::ID, false)
        ]
    );
    let message: Message = Message::new(&[initialize_ix], Some(&payer_pkey));
    let mut tx: Transaction = Transaction::new_unsigned(message);

    // 4. get blockhash & sign tx & send init tx
    let blockhash: Hash = rpc_client.get_latest_blockhash().await?;
    log::info!("Got latest blockhash!");
    tx.sign(&[&payer], blockhash);

    send_tx_and_print_result(&rpc_client, &tx).await?;

    // 5. create increment_counter instruction & transaction
    let mut ix_payload: Vec<u8> = vec![1];
    let increment_by: u64 = 101;
    ix_payload.extend_from_slice(&increment_by.to_le_bytes());
    
    let increment_ix: Instruction = Instruction::new_with_bytes(
        counter::ID, 
        &ix_payload, 
        vec![
            AccountMeta::new(payer_pkey, true),
            AccountMeta::new(pda, false)
        ]
    );
    let message: Message = Message::new(&[increment_ix], Some(&payer_pkey));
    let mut tx: Transaction = Transaction::new_unsigned(message);

    // 6. sign tx & send increment tx
    tx.sign(&[&payer], blockhash);
    send_tx_and_print_result(&rpc_client, &tx).await?;

    // 7. init recipient
    let recipient: Keypair = Keypair::new();
    let recipient_pkey: Pubkey = recipient.pubkey();

    // 8. create close_counter instruction & transaction
    let close_ix: Instruction = Instruction::new_with_bytes(
        counter::ID, 
        &[2], 
        vec![
            AccountMeta::new(payer_pkey, true),
            AccountMeta::new(pda, false),
            AccountMeta::new(recipient_pkey, false)
        ]
    );
    let message: Message = Message::new(
        &[close_ix], 
        Some(&payer_pkey)
    );
    let mut close_tx: Transaction = Transaction::new_unsigned(message);

    // 9. sign close tx & send it
    close_tx.sign(&[&payer], blockhash);
    send_tx_and_print_result(&rpc_client, &close_tx).await?;

    Ok(())
}

async fn init_payer(rpc_client: &RpcClient) -> Result<(Keypair, Pubkey), Box<dyn std::error::Error>> {
    Ok(if std::env::var("NEW_PAYER")?.parse::<bool>()? {
        // 1. create payer & request airdrop & wait until balance tops up
        let payer: Keypair = Keypair::new();
        let payer_pkey: Pubkey = payer.pubkey();
        let airdrop_amount: u64 = LAMPORTS_PER_SOL * 5;

        let airdrop_sig: Signature = rpc_client.request_airdrop(&payer_pkey, airdrop_amount).await?;
        log::info!("Sending airdrop to payer account!");
        
        loop {
            if rpc_client.confirm_transaction(&airdrop_sig).await? {
                log::info!("Received airdrop.");
                break;
            }
        }

        (payer, payer_pkey)
    } else {
        // 1. or use every time the same account with some SOL
        let seed_phrase: String = std::env::var("SEED_PHRASE")?;
        let payer: Keypair = Keypair::from_base58_string(&seed_phrase);
        let payer_pkey: Pubkey = payer.pubkey();

        (payer, payer_pkey)
    })
}

async fn send_tx_and_print_result(rpc_client: &RpcClient, tx: &Transaction) -> solana_rpc_client_api::client_error::Result<()> {
    log::info!("Sending transaction!");
    match rpc_client.send_and_confirm_transaction(tx).await {
        Ok(sig) => log::info!("{}", sig),
        Err(e) => log::error!("Error: {}", e)
    };
    Ok(())
}
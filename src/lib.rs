pub mod state;
pub mod processor;
pub mod entrypoint;
pub mod instruction;

solana_program::declare_id!("3ie3FnnMBYqS6nN6UQWDxs4FcrN2UsnrZPCWrnM5242z");

#[cfg(test)]
mod tests {
    use solana_sdk::{
        hash::Hash,
        pubkey::Pubkey,
        signer::{Signer, keypair::Keypair},
        system_program,
        message::Message,
        transaction::Transaction,
        instruction::{Instruction, AccountMeta}
    };
    use solana_program_test::{ProgramTest, processor, BanksClient, BanksClientError};

    #[tokio::test]
    async fn test_initialize_counter() -> Result<(), Box<dyn std::error::Error>> {
        let program_test: ProgramTest = ProgramTest::new(
            "counter",
            crate::ID,
             processor!(crate::entrypoint::process_instruction)
        );

        // 1. init RpcClient, Payer, get Latest Blockhash
        let (banks_client, payer, latest_blockhash) = program_test.start().await;
        let payer_pkey = payer.pubkey();

        // 2. derive PDA
        let (pda, _bump) = Pubkey::find_program_address(
            &[b"counter", payer_pkey.as_ref()], 
            &crate::ID
        );

        // 3 & 4. craft init ix & init tx; sign & send init tx
        init_counter(&banks_client, latest_blockhash, payer_pkey, &payer, pda).await?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_init_and_increment_counter() -> Result<(), Box<dyn std::error::Error>> {
        let program_test: ProgramTest = ProgramTest::new(
            "counter",
            crate::ID,
             processor!(crate::entrypoint::process_instruction)
        );

        // 1. init RpcClient, Payer, get Latest Blockhash
        let (banks_client, payer, latest_blockhash) = program_test.start().await;
        let payer_pkey = payer.pubkey();

        // 2. derive PDA
        let (pda, _bump) = Pubkey::find_program_address(
            &[b"counter", payer_pkey.as_ref()],
            &crate::ID
        );

        // 3 & 4. craft init ix & init tx; sign & send init tx
        init_counter(&banks_client, latest_blockhash, payer_pkey, &payer, pda).await?;

        // // increment counter
        // // 5. create instruction & transaction
        let mut ix_payload: Vec<u8> = vec![1];  // 1 - is an instr_type, which is CounterInstruction::IncrementCounter
        let increment_by: u64 = 101; 
        ix_payload.extend_from_slice(&u64::to_le_bytes(increment_by));

        let increment_ix: Instruction = Instruction::new_with_bytes(
            crate::ID, 
            &ix_payload,
            vec![
                AccountMeta::new(payer_pkey, true),
                AccountMeta::new(pda, false),
            ]
        );
        let message: Message = Message::new(&[increment_ix], Some(&payer_pkey));
        let mut increment_tx: Transaction = Transaction::new_unsigned(message);

        // 6. sign tx & send tx
        increment_tx.sign(&[&payer], latest_blockhash);
        banks_client.process_transaction(increment_tx).await?;

        if let Ok(Some(account)) = banks_client.get_account(pda).await {
            let (value, is_initialized) = account.data.split_at(8);
            println!("Counter.value: {:?}", u64::from_le_bytes(value.try_into()?));
            println!("Counter.is_initialized: {:?}", if is_initialized[0] == 0 { false } else { true });
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_init_and_close_counter() -> Result<(), Box<dyn std::error::Error>> {
        let program_test: ProgramTest = ProgramTest::new(
            "counter", 
            crate::ID,
            processor!(crate::entrypoint::process_instruction) 
        );

        // 1. init client, payer, get hash 
        let (banks_client, payer, latest_blockhash) = program_test.start().await;
        let payer_pkey: Pubkey = payer.pubkey();

        // 2. derive PDA
        let (pda, _bump) = Pubkey::find_program_address(
            &[b"counter", payer_pkey.as_ref()],
            &crate::ID
        );

        // 3 & 4. craft init ix & init tx; sign & send init tx
        init_counter(&banks_client, latest_blockhash, payer_pkey, &payer, pda).await?;

        // close counter
        // 5. create recipient
        let recipient: Keypair = Keypair::new();
        let recipient_pkey: Pubkey = recipient.pubkey();

        // 6. craft close ix & close tx
        let close_ix: Instruction = Instruction::new_with_bytes(
            crate::ID, 
            &[2],  // 2 - instr_type, which is CounterInstruction::CloseCounter 
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

        // 7. sign tx and send it
        close_tx.sign(&[&payer], latest_blockhash);
        banks_client.process_transaction(close_tx).await?;

        // 8. (optional) check recipient balance (don't forget to run cargo test-sbf with `-- --nocapture` flag to see prints)
        match banks_client.get_balance(recipient_pkey).await {
            Ok(balance) => println!("recipient balance: {}", balance),
            Err(e) => println!("Get recipient balance returned error: {}", e)
        }

        Ok(())
    }

    /// Invokes `initialize_counter` instruction.
    async fn init_counter(
        banks_client: &BanksClient,
        latest_blockhash: Hash,
        payer_pkey: Pubkey,
        payer: &Keypair,
        pda: Pubkey,
    ) -> Result<(), BanksClientError> {
        let initialize_ix: Instruction = Instruction::new_with_bytes(
            crate::ID, 
            &[0],  // 0 is an instr_type, which is CounterInstruction::InitializeCounter
            vec![
                AccountMeta::new(payer_pkey, true),
                AccountMeta::new(pda, false),
                AccountMeta::new_readonly(system_program::ID, false)
            ]
        );
        let message: Message = Message::new(&[initialize_ix], Some(&payer_pkey));
        let mut init_tx: Transaction = Transaction::new_unsigned(message);
        
        // sign tx & send tx
        init_tx.sign(&[payer], latest_blockhash);
        banks_client.process_transaction(init_tx).await?;
        
        Ok(())
    }
}
pub mod state;
pub mod processor;
pub mod entrypoint;
pub mod instruction;

solana_program::declare_id!("3ie3FnnMBYqS6nN6UQWDxs4FcrN2UsnrZPCWrnM5242z");

#[cfg(test)]
mod tests {
    use solana_sdk::{
        pubkey::Pubkey,
        signer::Signer,
        system_program,
        message::Message,
        transaction::Transaction,
        instruction::{Instruction, AccountMeta}
    };
    use solana_program_test::{ProgramTest, processor};

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

        // 3. create instruction & transaction
        let initialize_ix: Instruction = Instruction::new_with_bytes(
            crate::ID, 
            &[0],  // 0 is an instr_type, which is CounterInstruction::InitializeCounter
            vec![
                AccountMeta::new(payer_pkey.clone(), true),
                AccountMeta::new(pda, false),
                AccountMeta::new_readonly(system_program::ID, false)
            ]
        );
        let message: Message = Message::new(&[initialize_ix], Some(&payer_pkey));
        let mut tx: Transaction = Transaction::new_unsigned(message);

        // 4. sign tx & send tx
        tx.sign(&[&payer], latest_blockhash);
        banks_client.process_transaction(tx).await?;

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

        // 3. create instruction & transaction
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

        // 4. sign tx & send tx
        init_tx.sign(&[&payer], latest_blockhash);
        banks_client.process_transaction(init_tx).await?;

        // // increment counter
        // // 1. create instruction & transaction
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

        // 2. sign tx & send tx
        increment_tx.sign(&[&payer], latest_blockhash);
        banks_client.process_transaction(increment_tx).await?;

        if let Ok(Some(account)) = banks_client.get_account(pda).await {
            let (value, is_initialized) = account.data.split_at(8);
            println!("Counter.value: {:?}", u64::from_le_bytes(value.try_into()?));
            println!("Counter.is_initialized: {:?}", if is_initialized[0] == 0 { false } else { true });
        }

        Ok(())
    }
}
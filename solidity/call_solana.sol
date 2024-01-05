// SPDX-License-Identifier: MIT

pragma solidity >= 0.7.0;
pragma abicoder v2;

interface CallSolana {

    // Returns Solana address for Neon address.
    // Calculates as PDA([ACCOUNT_SEED_VERSION, Neon-address], evm_loader_id)
    function getNeonAddress(address) external returns (bytes32);

    // Returns Solana address of resource for contracts.
    // Calculates as PDA([ACCONT_SEED_VERSION, "ContractData", msg.sender, salt], evm_loader_id)
    function getResourceAddress(bytes32 salt) external returns (bytes32);

    // Creates resource with specified salt.
    // Return the Solana address of created resource (see `getResourceAddress`)
    function createResource(bytes32 salt, uint64 space, uint64 lamports, bytes32 owner) external returns (bytes32);

    // Returns Solana PDA generated from specified program_id and seeds
    function getSolanaPDA(bytes32 program_id, bytes memory seeds) external returns (bytes32);

    // Returns Solana address of external authority.
    // Calculates as PDA([ACCOUNT_SEED_VERSION, "AUTH", msg.sender, salt], evm_loader_id)
    function getExtAuthority(bytes32 salt) external returns (bytes32);

    // Return Solana address for payer account (if instruction required some account to funding new created accounts)
    // Calculates as PDA([ACCOUNT_SEED_VERSION, "PAYER", msg.sender], evm_loader_id)
    function getPayer() external returns (bytes32);

    // Perform call to the Solana program with `program_id`.
    // Pass a list of accounts and data
    // Garantees success execution of call after return.
    // Note: If call was unsucessful, the transaction fails (due to Solana behaviour).
    //function call(bytes32 program_id, AccountInfo[] memory accounts, bytes memory data, uint64 lamports) external;
    //function call2(bytes32 program_id, AccountInfo[2] memory accounts, bytes memory data, uint64 lamports) external;

    // Execute the instruction with call to the Solana program.
    // - `lamports` specifes amount of lamports that can be required to create new accounts during execution.
    //   This lamports transferred to `payer`-account (see `getPayer()` function) before the call.
    // - `instruction` - serialized instruction which should be executed
    // This method uses PDA for sender to authorize the operation (`getNeonAddress(msg.sender)`)
    function execute(uint64 lamports, bytes memory instruction) external;

    // Execute the instruction with call to the Solana program.
    // - `lamports` specifes amount of lamports that can be required to create new accounts during execution.
    //   This lamports transferred to `payer`-account (see `getPayer()` function) before the call.
    // - `salt` - the salt to generate address of external authority (see `getExtAuthority()` function)
    // - `instruction` - serialized instruction which should be executed
    // This method uses external authority to authorize the operation (`getExtAuthority(salt)`)
    function executeWithSeed(uint64 lamports, bytes32 salt, bytes memory instruction) external;
}

/* Note:
Instruction should be serialized according to Solana bincode serialize rules. It requires
serialized data in next form:
    program_id as bytes32
    len(accounts) as uint64le
        account as bytes32
        is_signer as bool
        is_writable as bool
    len(data) as uint64le
        data (see instruction to Solana program)

The optimized way to serailize instruction is write this code on the solidity assembler.
To perform a call to `execute()` and `executeWithSeed()` methods the next code-sample can be helpful:
```solidity
    {
        // TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        bytes32 program_id = 0x06ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a9;
        bytes32 owner = getNeonAddress(address(this));

        bytes4 selector = bytes4(keccak256("execute(uint64,bytes)"));
        bool success;
        assembly {
            let buff := mload(0x40)    // the head of heap
            let pos := buff            // current write position
            
            // selector
            mstore(pos, selector)      // write the method selector
            pos := add(pos, 4)

            // Write arguments to call the method
            // lamports
            mstore(pos, 0)             // write required lamports
            pos := add(pos, 32)

            // offset for instruction
            // specify the position of serialized instruction relative to start of arguments
            mstore(pos, sub(add(pos, 28), buff))
            pos := add(pos, 32)
            let size_pos := pos        // Save size position of serialized instruction
            pos := add(pos, 32)

            // program_id
            mstore(pos, program_id)
            pos := add(pos, 32)

            // len(accounts)
            mstore(pos, 0)
            mstore8(pos, 4)
            pos := add(pos, 8)

            // For each account in accounts array:
                // AccountMeta(resource, false, true)
                mstore(pos, owner)        // pubkey
                mstore8(add(pos, 32), 1)  // is_signer
                mstore8(add(pos, 33), 0)  // is_writable
                pos := add(pos, 34)

            // len(instruction_data)  if it shorter than 256 bytes
            mstore(pos, 0)            // fill with zero next 32 bytes
            mstore8(pos, 1)           // write the length of data
            pos := add(pos, 8)

            // instruction_data: InitializeAccount
            mstore8(pos, 1)           // Use Solana program instruction to detailed info
            pos := add(pos, 1)

            mstore(size_pos, sub(sub(pos, size_pos), 32))  // write the size of serialized instruction 
            let length := sub(pos, buff)      // calculate the length of arguments
            mstore(0x40, pos)                 // update head of heap
            success := call(5000, 0xFF00000000000000000000000000000000000006, 0, buff, length, buff, 0x20)
            mstore(0x40, buff)                // restore head of heap
        }
        if (success == false) {
            revert("Can't initailize resource");
        }
    }
```
*/
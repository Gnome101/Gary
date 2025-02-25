// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.13;

import "dependencies/forge-std-1.9.4/src/Script.sol";
import "../contracts/src/IncredibleSquaringTaskManager.sol";

contract CreateTaskScript is Script {
    // Default Anvil account's private key (for testing on Anvil)
    uint256 constant DEFAULT_ANVIL_PRIVATE_KEY =
        0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80;

    function run() external {
        // Use the default Anvil account as the generator.

        vm.startBroadcast(DEFAULT_ANVIL_PRIVATE_KEY);

        // The deployed contract address.
        address taskManagerAddress = 0xD0141E899a65C95a556fE2B27e5982A6DE7fDD7A;
        IncredibleSquaringTaskManager taskManager = IncredibleSquaringTaskManager(
                taskManagerAddress
            );

        // Define task parameters.
        uint256 numberToBeSquared = 9; // Example value.
        uint32 quorumThresholdPercentage = 75; // Example threshold.
        bytes memory quorumNumbers = ""; // Empty placeholder; adjust as needed.

        // Call createNewTask.
        taskManager.createNewTask(
            numberToBeSquared,
            quorumThresholdPercentage,
            quorumNumbers
        );

        // Retrieve and log the latest task number.
        uint32 latestTaskNum = taskManager.taskNumber();
        console.log("Latest Task Number:", latestTaskNum);

        vm.stopBroadcast();
    }
}

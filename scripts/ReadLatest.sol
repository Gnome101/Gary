// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.13;

import "dependencies/forge-std-1.9.4/src/Script.sol";
import "../contracts/src/IncredibleSquaringTaskManager.sol";

contract CreateTaskScript is Script {
    // Default Anvil account's private key (for testing on Anvil)
    uint256 constant DEFAULT_ANVIL_PRIVATE_KEY =
        0xac0974bec39a17e36ba4a6ac5181e01e65cd26d6f3e2d0b9a0e4239c8fcf4e99;

    function run() external {
        // Use the default Anvil account as the generator.
        vm.startBroadcast(DEFAULT_ANVIL_PRIVATE_KEY);

        // The deployed contract address.
        address taskManagerAddress = 0xD0141E899a65C95a556fE2B27e5982A6DE7fDD7A;
        IncredibleSquaringTaskManager taskManager = IncredibleSquaringTaskManager(
                taskManagerAddress
            );

        // Retrieve and log the latest task number.
        (uint32 x, uint256 num) = taskManager.latestResponse();
        uint256 c = taskManager.c();

        console.log("Latest Task Number:", x, num, c);

        vm.stopBroadcast();
    }
}

// SPDX-License-Identifier: Unlicense
pragma solidity ^0.8.15;

import "foundry-huff/HuffDeployer.sol";
import "forge-std/Script.sol";

contract DeploySando is Script {
    function run() public returns (address sandoAddress) {
        sandoAddress = HuffDeployer.deploy("Sando");
    }
}

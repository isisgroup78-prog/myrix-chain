// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {ERC20Burnable} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";

/// @title MYRIX ecosystem token
/// @notice Fixed-supply ERC-20 for EVM applications built around MYRIX.
/// @dev This contract is an application-layer token, not the native MYRIX coin.
contract MyrixToken is ERC20, ERC20Burnable {
    uint256 public constant INITIAL_SUPPLY = 1_000_000_000 ether;

    constructor(address treasury) ERC20("MYRIX", "MYX") {
        require(treasury != address(0), "treasury is zero");
        _mint(treasury, INITIAL_SUPPLY);
    }
}

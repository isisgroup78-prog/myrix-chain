// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {MyrixToken} from "../src/MyrixToken.sol";

contract MyrixTokenTest is Test {
    MyrixToken token;
    address treasury = address(0xBEEF);
    address alice = address(0xA11CE);

    function setUp() public {
        token = new MyrixToken(treasury);
    }

    function testInitialSupply() public {
        assertEq(token.totalSupply(), 1_000_000_000 ether);
        assertEq(token.balanceOf(treasury), 1_000_000_000 ether);
    }

    function testTransferAndBurn() public {
        vm.prank(treasury);
        token.transfer(alice, 100 ether);

        assertEq(token.balanceOf(alice), 100 ether);

        vm.prank(alice);
        token.burn(40 ether);

        assertEq(token.balanceOf(alice), 60 ether);
        assertEq(token.totalSupply(), 999_999_940 ether);
    }

    function testZeroTreasuryRejected() public {
        vm.expectRevert(bytes("treasury is zero"));
        new MyrixToken(address(0));
    }
}

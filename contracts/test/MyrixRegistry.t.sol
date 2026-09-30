// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {MyrixRegistry} from "../src/MyrixRegistry.sol";

contract MyrixRegistryTest is Test {
    MyrixRegistry registry;
    address owner = address(0x1234);
    bytes32 key = keccak256("myrix.app");

    function setUp() public {
        registry = new MyrixRegistry(owner);
    }

    function testSetAndReadRecord() public {
        vm.prank(owner);
        registry.setRecord(key, bytes("MYRIX"));

        assertEq(registry.getRecord(key), bytes("MYRIX"));
    }

    function testUnauthorizedWriteRejected() public {
        vm.expectRevert();
        registry.setRecord(key, bytes("blocked"));
    }
}

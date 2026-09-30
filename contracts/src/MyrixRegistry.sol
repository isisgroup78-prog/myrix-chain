// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {Ownable2Step} from "@openzeppelin/contracts/access/Ownable2Step.sol";

/// @title MYRIX application registry
/// @notice Minimal on-chain registry for protocol/application metadata.
contract MyrixRegistry is Ownable2Step {
    mapping(bytes32 => bytes) private _records;

    event RecordSet(bytes32 indexed key, bytes value);
    event RecordDeleted(bytes32 indexed key);

    constructor(address initialOwner) Ownable(initialOwner) {}

    function setRecord(bytes32 key, bytes calldata value) external onlyOwner {
        _records[key] = value;
        emit RecordSet(key, value);
    }

    function getRecord(bytes32 key) external view returns (bytes memory) {
        return _records[key];
    }

    function deleteRecord(bytes32 key) external onlyOwner {
        delete _records[key];
        emit RecordDeleted(key);
    }
}

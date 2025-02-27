// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.13;

import "@openzeppelin-upgrades/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgrades/contracts/access/OwnableUpgradeable.sol";
import "./IIncredibleSquaringTaskManager.sol";
import {BLSSignatureChecker, IRegistryCoordinator} from "eigenlayer-middleware/src/BLSSignatureChecker.sol";

/**
 * For clarity, we remove the BLS code from the example.
 * In practice, you can keep your existing fields / logic
 * and simply add these new fields/events.
 */
contract IncredibleSquaringTaskManager is
    Initializable,
    OwnableUpgradeable,
    IIncredibleSquaringTaskManager
{
    // ==============
    //   NEW STORAGE
    // ==============

    /// @dev Aggregator’s public key for ElGamal, stored on chain so users can encrypt.
    /// For Ristretto, you typically have 2 group elements.
    /// We store them concatenated as 64 bytes (2 × 32).
    bytes public aggregatorPublicKey;

    /// @dev Each ciphertext is two group elements: (c1, c2).
    struct ElGamalCiphertext {
        bytes c1; // compressed Ristretto point
        bytes c2; // compressed Ristretto point
    }

    /// @dev Array of all ciphertext submissions (appended by `submitEncryptedValue`).
    ElGamalCiphertext[] public allCiphertexts;

    /// @dev The final decrypted sum (once aggregator calls setDecryptionResult).
    uint256 public finalDecryptedValue;
    bool public isDecryptionSet;

    /// @dev If a decryption request is active.
    bool public isDecryptionRequested;

    // ==============
    //   EVENTS
    // ==============

    // Other events from your original contract
    // e.g., event NewTaskCreated(...), etc.

    // ==============
    //   INITIALIZER
    // ==============
    function initialize(address initialOwner) public initializer {
        __Ownable_init();
        _transferOwnership(initialOwner);
        // Optionally set aggregatorPublicKey if you already have it
    }

    // ==============
    //   PUBLIC KEY
    // ==============

    /// @dev Owner sets aggregator's ElGamal public key.
    /// Typically done once, or if aggregator changes keys.
    function setAggregatorPublicKey(bytes calldata pk) external onlyOwner {
        // For Ristretto, you expect pk to be 64 bytes (2 × 32)
        require(pk.length == 64, "Invalid PK length");
        aggregatorPublicKey = pk;
        emit AggregatorPublicKeySet(pk);
    }

    // ==============
    //   SUBMISSIONS
    // ==============

    /// @dev Store a user's ciphertext.
    /// Each c1,c2 is typically 32 bytes (compressed Ristretto).
    function submitEncryptedValue(
        bytes calldata c1,
        bytes calldata c2
    ) external {
        // Basic checks
        require(c1.length == 32, "Invalid c1 length");
        require(c2.length == 32, "Invalid c2 length");

        // Save to array
        allCiphertexts.push(ElGamalCiphertext({c1: c1, c2: c2}));
        emit EncryptedValueSubmitted(msg.sender, c1, c2);
    }

    /// @dev Number of submissions so aggregator can retrieve them.
    function ciphertextCount() external view returns (uint256) {
        return allCiphertexts.length;
    }

    // ==============
    //  DECRYPTION
    // ==============

    /// @dev Anyone can request aggregator to finalize
    function requestDecryption() external {
        require(!isDecryptionRequested, "Already requested");
        isDecryptionRequested = true;
        emit DecryptionRequested(msg.sender);
    }

    /// @dev Aggregator (owner in this example) sets final plaintext sum on chain
    function setDecryptionResult(uint256 sum) external onlyOwner {
        require(isDecryptionRequested, "Not requested yet");
        require(!isDecryptionSet, "Already finalized");

        finalDecryptedValue = sum;
        isDecryptionSet = true;
        emit DecryptionResultPosted(sum);
    }

    function respondToTask(
        Task calldata task,
        TaskResponse calldata taskResponse
    ) external {}
    // ==============
    //  [Existing BLS code or other tasks go below...]
    // ==============
}

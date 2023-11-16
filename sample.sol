// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/Counters.sol";

contract TimeableMergeableNFT is ERC721, Ownable {
    using Counters for Counters.Counter;
    Counters.Counter private _tokenIds;

    mapping(uint256 => uint256) private mergeTimes;

    constructor(string memory name, string memory symbol) ERC721(name, symbol) {}

    function mint(address to, uint256 mergeAfterSeconds) external onlyOwner {
        _tokenIds.increment();
        uint256 tokenId = _tokenIds.current();
        _safeMint(to, tokenId);
        mergeTimes[tokenId] = block.timestamp + mergeAfterSeconds;
    }

    function canMerge(uint256 tokenId) public view returns (bool) {
        return block.timestamp >= mergeTimes[tokenId];
    }

    function merge(uint256 tokenId1, uint256 tokenId2) external {
        require(canMerge(tokenId1) && canMerge(tokenId2), "Not yet mergeable");
        require(_isApprovedOrOwner(msg.sender, tokenId1) && _isApprovedOrOwner(msg.sender, tokenId2), "Not approved or not owner");
        _burn(tokenId1);
        _burn(tokenId2);

        // Burada, iki NFT'yi birleştirerek yeni bir NFT oluşturabilirsiniz.
        // Bu işlemi gerçekleştirecek özel bir mantık eklemelisiniz.
        // Örneğin, yeni bir NFT oluşturarak `_safeMint` fonksiyonunu çağırabilirsiniz.
    }
}
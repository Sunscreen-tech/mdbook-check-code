# Solidity Example

This file tests Solidity contract compilation.

## Simple Storage Contract

Here's a basic storage contract:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract SimpleStorage {
    uint256 private value;

    function set(uint256 newValue) public {
        value = newValue;
    }

    function get() public view returns (uint256) {
        return value;
    }
}
```

## Using sol alias

The `sol` alias should also work:

```sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Counter {
    uint256 public count;

    function increment() public {
        count += 1;
    }
}
```

## Ignored code

```solidity,ignore
This should not compile!
invalid solidity syntax
```

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.15;

import "./GeneralHelper.sol";
import "./SandoCommon.sol";

/// @title V2SandoUtility
/// @author 0xmouseless
/// @notice Functions for interacting with sando contract's v2 methods
library V2SandoUtility {
    /**
     * @notice Utility function to create payload for our v2 backruns
     * @return payload Calldata bytes to execute backruns
     * @return encodedValue Encoded `tx.value` indicating WETH amount to send
     */
    function v2CreateBackrunPayload(
        address otherToken,
        uint256 amountIn
    ) public view returns (bytes memory payload, uint256 encodedValue) {
        // Declare uniswapv2 types
        address weth = address(0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c);
        IUniswapV2Factory univ2Factory = IUniswapV2Factory(
            0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73
        );
        address pair = address(
            IUniswapV2Pair(univ2Factory.getPair(weth, address(otherToken)))
        );

        // encode amountIn
        FiveBytesEncodingUtils.EncodingMetaData
            memory fiveByteParams = FiveBytesEncodingUtils.encode(amountIn);
        uint256 amountInActual = FiveBytesEncodingUtils.decode(fiveByteParams);

        string memory functionSignature = weth < otherToken
            ? "v2_backrun0"
            : "v2_backrun1";
        uint8 jumpDest = SandoCommon.getJumpDestFromSig(functionSignature);

        payload = abi.encodePacked(
            jumpDest,
            address(pair), // univ2 pair
            address(otherToken), // inputToken
            FiveBytesEncodingUtils.finalzeForParamIndex(fiveByteParams, 1)
        );

        uint256 amountOut = GeneralHelper.getAmountOut(
            otherToken,
            weth,
            amountInActual
        );
        encodedValue = WethEncodingUtils.encode(amountOut);
    }

    /**
     * @notice Utility function to create payload for our v2 frontruns
     * @return payload Calldata bytes to execute frontruns
     * @return encodedValue Encoded `tx.value` indicating WETH amount to send
     */
    function v2CreateFrontrunPayload(
        address outputToken,
        uint256 amountIn
    ) public view returns (bytes memory payload, uint256 encodedValue) {
        // Declare uniswapv2 types
        address weth = address(0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c);
        IUniswapV2Factory univ2Factory = IUniswapV2Factory(
            0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73
        );
        address pair = address(
            IUniswapV2Pair(univ2Factory.getPair(weth, address(outputToken)))
        );

        // Encode amountIn here (so we can use it for next step)
        uint256 amountInActual = WethEncodingUtils.decode(
            WethEncodingUtils.encode(amountIn)
        );

        // Get amounts out and encode it
        FiveBytesEncodingUtils.EncodingMetaData
            memory fiveByteParams = FiveBytesEncodingUtils.encode(
                GeneralHelper.getAmountOut(weth, outputToken, amountInActual)
            );

        string memory functionSignature = weth < outputToken
            ? "v2_frontrun0"
            : "v2_frontrun1";
        uint8 jumpDest = SandoCommon.getJumpDestFromSig(functionSignature);

        payload = abi.encodePacked(
            jumpDest, // type of swap to make
            address(pair), // univ2 pair
            FiveBytesEncodingUtils.finalzeForParamIndex(
                fiveByteParams,
                weth < outputToken ? 1 : 0
            )
        );

        encodedValue = WethEncodingUtils.encode(amountIn);
    }
}

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "forge-std/Test.sol";
import "forge-std/console2.sol";
import "v2-core/interfaces/IUniswapV2Pair.sol";
import "v2-core/interfaces/IUniswapV2Factory.sol";
import "v2-periphery/interfaces/IUniswapV2Router02.sol";
import "v3-periphery/interfaces/IQuoter.sol";
import "v3-core/interfaces/IUniswapV3Pool.sol";
import "solmate/tokens/WETH.sol";

import "../src/LilRouter.sol";

/// @title LilRouterTest
/// @author 0xmouseless
/// @notice Test suite for the LilRouter contract
contract LilRouterTest is Test {
    address constant weth = 0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c;
    IUniswapV2Factory uniV2Factory;
    IUniswapV2Router02 uniV2Router;
    IQuoter uniV3Quoter;
    LilRouter lilRouter;

    /// @notice Set up the testing suite
    function setUp() public {
        lilRouter = new LilRouter();

        uniV2Factory = IUniswapV2Factory(
            0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73
        );
        uniV2Router = IUniswapV2Router02(
            0x10ED43C718714eb63d5aA57B78B54704E256024E
        );
        uniV3Quoter = IQuoter(0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6);
        WETH wrappedEther = WETH(payable(weth));

        wrappedEther.deposit{value: 10e18}();
        wrappedEther.transfer(address(lilRouter), 10e18);
    }

    /// @notice Test swapping weth to usdc and back
    function testUniswapV3() public {
        address usdc = 0x55d398326f99059fF775485246999027B3197955;
        address usdcWethPool = 0x36696169C63e42cd08ce11f5deeBbCeBae652050; // 500 fee pool

        address testToken = 0xd98438889Ae7364c7E2A3540547Fad042FB24642;
        address testTokenPool = 0xA2C1e0237bF4B58bC9808A579715dF57522F41b2;

        // swapping 2 weth to usdc
        int256 amountIn = 4600000000000000000;
        /*uint256 amountOutExpected = _quoteV3Swap(
            amountIn,
            usdcWethPool,
            weth,
            usdc
        );*/
        (uint256 amountOut, uint256 realAfterBalance) = lilRouter
            .calculateSwapV3(amountIn, testTokenPool, weth, testToken);
        console2.log(
            "swapped %d WBNB for %d CELL",
            uint256(amountIn),
            amountOut
        );
        console2.log("realAfterBalance %d", realAfterBalance);
        /*assertEq(
            amountOutExpected,
            amountOut,
            "WETH->USDC swap failed: received USDC deviates from expected router output."
        );*/

        // swapping received usdc back to weth
        /*amountIn = int256(amountOut);
        //amountOutExpected = _quoteV3Swap(amountIn, usdcWethPool, usdc, weth);
        (amountOut, ) = lilRouter.calculateSwapV3(
            amountIn,
            usdcWethPool,
            usdc,
            weth
        );
        console2.log(
            "swapped %d USDC for %d WETH",
            uint256(amountIn),
            amountOut
        );*/
        /*assertEq(
            amountOutExpected,
            amountOut,
            "USDC->WETH swap failed: received WETH deviates from expected router output."
        );*/
    }

    /// @notice Test swapping weth to usdc and back
    function testUniswapV2() public {
        address usdc = 0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d;
        address usdcWethPair = _getPairUniV2(usdc, address(weth));

        // swapping 2 weth to usdc
        uint256 amountIn = 2;
        uint256 amountOutExpected = _quoteV2Swap(
            amountIn,
            usdcWethPair,
            weth < usdc
        );
        (uint256 amountOut, ) = lilRouter.calculateSwapV2(
            amountIn,
            usdcWethPair,
            weth,
            usdc
        );
        console2.log("swapped %d WETH for %d USDC", amountIn, amountOut);
        assertEq(
            amountOutExpected,
            amountOut,
            "WETH->USDC swap failed: received USDC deviates from expected router output."
        );

        // swapping received usdc back to weth
        amountIn = amountOut;
        amountOutExpected = _quoteV2Swap(amountIn, usdcWethPair, usdc < weth);
        (amountOut, ) = lilRouter.calculateSwapV2(
            amountIn,
            usdcWethPair,
            usdc,
            weth
        );
        console2.log("swapped %d USDC for %d WETH", amountIn, amountOut);
        assertEq(
            amountOutExpected,
            amountOut,
            "USDC->WETH swap failed: received WETH deviates from expected router output."
        );
    }

    /// @notice Get the deployed LilRouter bytecode (we inject this into evm instances for simulations)
    function testGetLilRouterCode() public {
        bytes memory code = address(lilRouter).code;
        emit log_bytes(code);
    }

    // -------------
    // -- HELPERS --
    // -------------
    function _quoteV3Swap(
        int256 amountIn,
        address _pool,
        address tokenIn,
        address tokenOut
    ) private returns (uint256 amountOut) {
        IUniswapV3Pool pool = IUniswapV3Pool(_pool);

        // wether tokenIn is token0 or token1
        bool zeroForOne = tokenIn < tokenOut;
        // From docs: The Q64.96 sqrt price limit. If zero for one,
        // The price cannot be less than this value after the swap.
        // If one for zero, the price cannot be greater than this value after the swap
        uint160 sqrtPriceLimitX96 = (
            zeroForOne
                ? 4295128749
                : 1461446703485210103287273052203988822378723970341
        );

        amountOut = uniV3Quoter.quoteExactInputSingle(
            tokenIn,
            tokenOut,
            pool.fee(),
            uint256(amountIn),
            sqrtPriceLimitX96
        );
    }

    function _quoteV2Swap(
        uint256 amountIn,
        address pair,
        bool isInputToken0
    ) private view returns (uint256 amountOut) {
        (uint256 reserveIn, uint256 reserveOut, ) = IUniswapV2Pair(pair)
            .getReserves();

        if (!isInputToken0) {
            // reserveIn is token1
            (reserveIn, reserveOut) = (reserveOut, reserveIn);
        }

        amountOut = uniV2Router.getAmountOut(amountIn, reserveIn, reserveOut);
    }

    function _getPairUniV2(
        address tokenA,
        address tokenB
    ) private view returns (address pair) {
        pair = uniV2Factory.getPair(tokenA, tokenB);
    }
}

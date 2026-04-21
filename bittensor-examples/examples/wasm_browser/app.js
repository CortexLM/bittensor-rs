/**
 * Standalone JS example showing how to use bittensor-wasm from a browser.
 *
 * Build the WASM package first:
 *   wasm-pack build bittensor-wasm --target web
 *
 * Then serve this directory with any static file server:
 *   python3 -m http.server 3180
 *
 * Open http://localhost:3180/index.html
 */

import init, { Balance, AxonInfo, get_balance } from '../../pkg/bittensor_wasm.js';

async function run() {
    await init();

    // Create a Balance from TAO
    const bal = Balance.from_tao("2.5");
    console.log("Balance:", bal.display());

    // Parse an AxonInfo from JSON
    const info = AxonInfo.from_json(
        JSON.stringify({
            ip: "10.0.0.1",
            port: 8091,
            ip_type: 4,
            protocol: 4,
            version: 0,
            hotkey: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
            coldkey: "5FHwWtKnMnGrnEtGBfcdTrpRVn2g2vG3oGK4rPfYJucr4ULn",
        })
    );
    console.log("Hotkey:", info.hotkey());
    console.log("Port:", info.port());
    console.log("JSON:", info.to_json());

    // Call get_balance (requires live chain connection — will fail without one)
    try {
        const result = await get_balance("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
        console.log("Chain balance:", result);
    } catch (e) {
        console.log("get_balance skipped (no chain connection):", e);
    }
}

run();

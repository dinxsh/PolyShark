/**
 * PolyShark Gator Bridge Example
 * 
 * This demonstrates how to use MetaMask's Smart Accounts Kit / Delegation Toolkit
 * to request an ERC-7715 permission, then hand off to the PolyShark Rust agent.
 * 
 * This is a reference implementation showing ecosystem integration.
 * In production, you would run this in a browser context with MetaMask.
 * 
 * @see https://docs.metamask.io/smart-accounts/delegation-toolkit
 * @see https://github.com/MetaMask/create-gator-app
 */

// Permission configuration matching PolyShark's Rust implementation
const POLYSHARK_PERMISSION = {
  "erc7715:permission": {
    type: "spend",
    token: {
      symbol: "USDC",
      address: "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174", // Polygon USDC
    },
    limit: {
      amount: 10.0,
      period: "day",
    },
    duration: {
      days: 30,
    },
    scope: {
      protocol: "polymarket",
      adapter: "polyshark",
    },
    metadata: {
      title: "PolyShark Trading Permission",
      description:
        "PolyShark may automatically trade up to 10 USDC per day on your behalf for 30 days.",
    },
  },
};

/**
 * Request ERC-7715 permission from MetaMask
 * 
 * In a real implementation using create-gator-app, you would use:
 * - @metamask/delegation-toolkit for delegation management
 * - Smart Account session keys for execution
 */
async function requestPolySharkPermission(config: {
  dailyLimit: number;
  durationDays: number;
  token: string;
}): Promise<{ permissionId: string; sessionKey: string }> {
  // Build permission object with user config
  const permission = {
    ...POLYSHARK_PERMISSION,
    "erc7715:permission": {
      ...POLYSHARK_PERMISSION["erc7715:permission"],
      limit: {
        amount: config.dailyLimit,
        period: "day",
      },
      duration: {
        days: config.durationDays,
      },
      token: {
        ...POLYSHARK_PERMISSION["erc7715:permission"].token,
        symbol: config.token,
      },
    },
  };

  console.log("📋 Requesting ERC-7715 Permission:");
  console.log(JSON.stringify(permission, null, 2));

  // In a real browser context with MetaMask:
  // const provider = window.ethereum;
  // const accounts = await provider.request({ method: 'eth_requestAccounts' });
  // const result = await provider.request({
  //   method: 'wallet_requestPermissions',
  //   params: [permission],
  // });

  // For this example, we simulate the response
  const mockResponse = {
    permissionId: `perm_${Date.now()}`,
    sessionKey: `0x${Array(64).fill(0).map(() => 
      Math.floor(Math.random() * 16).toString(16)
    ).join('')}`,
  };

  console.log("✅ Permission granted!");
  console.log(`   Permission ID: ${mockResponse.permissionId}`);
  console.log(`   Session Key: ${mockResponse.sessionKey.slice(0, 20)}...`);

  return mockResponse;
}

/**
 * Hand off to PolyShark Rust agent
 * 
 * The Rust agent will use the session key to execute trades
 * within the permission bounds.
 */
function handOffToRustAgent(sessionKey: string): void {
  console.log("\n🦈 Handing off to PolyShark Rust agent...");
  console.log("   The agent will now trade autonomously within permission bounds.");
  console.log("   No further wallet popups required!");
  
  // In production, you would:
  // 1. Store the session key securely
  // 2. Start the Rust agent with the session key
  // 3. The agent reads the key from config and uses it for execution
  
  // Example: spawn Rust process or send to backend
  // await fetch('/api/start-agent', { 
  //   method: 'POST', 
  //   body: JSON.stringify({ sessionKey }) 
  // });
}

/**
 * Main flow demonstrating the PolyShark permission lifecycle
 */
async function main(): Promise<void> {
  console.log("🦈 PolyShark Gator Bridge Example\n");
  console.log("This demonstrates ERC-7715 permission flow using MetaMask's ecosystem.\n");

  // Step 1: Request permission
  const { sessionKey } = await requestPolySharkPermission({
    dailyLimit: 10.0,
    durationDays: 30,
    token: "USDC",
  });

  // Step 2: Hand off to Rust agent
  handOffToRustAgent(sessionKey);

  console.log("\n✨ Done! The agent is now running autonomously.");
  console.log("   User can revoke permission at any time via MetaMask or dashboard.");
}

// Run the example
main().catch(console.error);

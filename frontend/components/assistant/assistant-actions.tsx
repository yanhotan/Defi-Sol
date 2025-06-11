"use server"

// This is a mock function that would be replaced with actual AI integration
export async function getAssistantResponse(userMessage: string): Promise<string> {
  try {
    // For a real implementation, you would use the AI SDK
    // const { text } = await generateText({
    //   model: openai("gpt-4o"),
    //   prompt: createPrompt(userMessage),
    // })
    // return text

    // For now, we'll use a mock implementation
    return mockAssistantResponse(userMessage)
  } catch (error) {
    console.error("Error generating assistant response:", error)
    return "I'm sorry, I encountered an error processing your request. Please try again."
  }
}

// Mock function to simulate AI responses
function mockAssistantResponse(userMessage: string): string {
  const message = userMessage.toLowerCase()

  // Check for pool-related questions
  if (message.includes("pool") && message.includes("difference")) {
    return `Our platform offers three different staking pools:

1. **Basic Pool**: Low-risk with instant unstaking, 5.2% APY
   - Best for beginners or those who need liquidity
   - No lock period, withdraw anytime
   - Lowest yield but highest flexibility

2. **Lending Pool**: Medium-risk with 7-day unstaking period, 7.8% APY
   - Your SOL is used for both staking and lending
   - Higher yields from borrower interest
   - 7-day unstaking period

3. **Lock Pool**: High-risk with adjustable lock periods, 12.5% to 25% APY
   - Highest yields with flexible lock periods (1, 3, 6, 9 months or 1 year)
   - Longer locks provide higher APY boosts (up to +100% for 1-year locks)
   - No early withdrawal option

Which pool are you most interested in learning more about?`
  }

  // Check for mSOL questions
  if (message.includes("msol") || message.includes("liquid staking")) {
    return `When you stake SOL in any of our pools, you receive mSOL tokens in return. These are liquid staking tokens that represent your staked SOL plus accumulated rewards.

Key benefits of mSOL:
- Transferable: You can send mSOL to other wallets
- Tradable: You can swap mSOL on DEXes
- Usable: You can use mSOL in other DeFi protocols while still earning staking rewards
- Value growth: mSOL gradually increases in value relative to SOL as staking rewards accumulate

The exchange rate between SOL and mSOL increases over time, reflecting your staking rewards.`
  }

  // Check for lending pool questions
  if (message.includes("lending pool") || message.includes("borrow")) {
    return `The Lending Pool works by allocating a portion of staked SOL to be borrowed by other users. Here's how it works:

1. **For Stakers (Lenders)**:
   - You stake SOL and receive mSOL
   - Your staked SOL earns both validator rewards (~5.2%) and lending interest (~2.6%)
   - Combined APY is currently 7.8%
   - 7-day unstaking period applies

2. **For Borrowers**:
   - Users can borrow SOL by providing collateral (150% collateralization ratio)
   - Current borrow APR is 9.2%
   - Dynamic interest rate model adjusts based on utilization
   - Liquidation occurs if collateral ratio falls below 120%

The current utilization rate is 68%, meaning 68% of the staked SOL in the lending pool is being borrowed.`
  }

  // Check for lock pool questions
  if (message.includes("lock pool") || message.includes("fixed term")) {
    return `The Lock Pool offers the highest yields in exchange for locking your SOL for a fixed period. You can choose your lock period using a simple slider:

- **1 Month (30 days)**: Base APY of 12.5%
- **3 Months (90 days)**: 15.63% APY (25% boost)
- **6 Months (180 days)**: 18.75% APY (50% boost)
- **9 Months (270 days)**: 21.88% APY (75% boost)
- **1 Year (365 days)**: 25% APY (100% boost)

Important considerations:
- You cannot withdraw early under any circumstances
- You still receive mSOL tokens, but they're marked as "locked" until the term ends
- Rewards are compounded and paid out at the end of the lock period
- After the lock period ends, you can withdraw, renew, or move to another pool

This pool is best for users who are confident they won't need access to their SOL during the lock period and want to maximize their returns.`
  }

  // Check for risk questions
  if (message.includes("risk") && message.includes("?")) {
    return `Each of our pools has different risk profiles:

**Basic Pool (Low Risk)**
- Validator slashing risk (mitigated by our validator selection)
- Smart contract risk (audited but not risk-free)
- No liquidity risk (instant unstaking)

**Lending Pool (Medium Risk)**
- All Basic Pool risks plus:
- Borrower default risk (mitigated by over-collateralization)
- Oracle price feed risk
- 7-day liquidity risk

**Lock Pool (High Risk)**
- All Basic Pool risks plus:
- Fixed term liquidity risk (no early withdrawal)
- Market volatility risk during lock period
- Higher smart contract complexity risk

We recommend diversifying across pools based on your risk tolerance and liquidity needs.`
  }

  // Default response
  return `I'm here to help with your staking questions. You can ask me about:

- Our different staking pools (Basic, Lending, and Lock)
- Liquid staking tokens (mSOL)
- APY rates and rewards
- Risks and benefits of each pool
- How to stake or unstake
- Lending and borrowing mechanics

What would you like to know more about?`
}

// This would be used with a real AI implementation
function createPrompt(userMessage: string): string {
  return `You are a helpful assistant for a Solana staking platform with three different pools:
  
1. Basic Pool: Low-risk, instant unstake, 5.2% APY
2. Lending Pool: Medium-risk, 7-day unstaking, 7.8% APY, lending enabled
3. Lock Pool: High-risk, adjustable lock periods (1, 3, 6, 9 months, 1 year), 12.5% to 25% APY

Users receive mSOL (liquid staking tokens) when they stake.
  
User message: ${userMessage}

Provide a helpful, accurate, and concise response about our staking pools. If the user is asking how to perform an action, provide step-by-step instructions. If they're asking about concepts, provide clear explanations with current data.`
}

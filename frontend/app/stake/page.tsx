import { PoolSelector } from "@/components/staking/pool-selector"
import { StakingForm } from "@/components/staking/staking-form"
import { StakingInfo } from "@/components/staking/staking-info"
import { PoolComparison } from "@/components/staking/pool-comparison"
import { WalletConnect } from "@/components/wallet-connect"

export default function StakePage() {
  return (
    <div className="flex flex-col gap-6 p-6 md:p-8">
      <div className="flex flex-col gap-2">
        <h1 className="text-3xl font-bold tracking-tight">Stake SOL</h1>
        <p className="text-muted-foreground">Stake your SOL tokens and earn rewards through different pool types</p>
      </div>
      <WalletConnect />
      <PoolSelector />
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <StakingForm />
        <StakingInfo />
      </div>
      <PoolComparison />
    </div>
  )
}

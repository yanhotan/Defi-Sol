import { RewardsChart } from "@/components/rewards/rewards-chart"
import { RewardsHistory } from "@/components/rewards/rewards-history"
import { RewardsStats } from "@/components/rewards/rewards-stats"
import { WalletConnect } from "@/components/wallet-connect"

export default function RewardsPage() {
  return (
    <div className="flex flex-col gap-6 p-6 md:p-8">
      <div className="flex flex-col gap-2">
        <h1 className="text-3xl font-bold tracking-tight">Rewards</h1>
        <p className="text-muted-foreground">Track your staking rewards and performance</p>
      </div>
      <WalletConnect />
      <RewardsStats />
      <RewardsChart />
      <RewardsHistory />
    </div>
  )
}

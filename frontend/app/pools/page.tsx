import { PoolStats } from "@/components/pools/pool-stats"
import { PoolsOverview } from "@/components/pools/pools-overview"
import { LendingMetrics } from "@/components/pools/lending-metrics"
import { WalletConnect } from "@/components/wallet-connect"

export default function PoolsPage() {
  return (
    <div className="flex flex-col gap-6 p-6 md:p-8">
      <div className="flex flex-col gap-2">
        <h1 className="text-3xl font-bold tracking-tight">Staking Pools</h1>
        <p className="text-muted-foreground">View detailed information about our staking pools</p>
      </div>
      <WalletConnect />
      <PoolStats />
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <PoolsOverview />
        <LendingMetrics />
      </div>
    </div>
  )
}

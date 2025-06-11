import { DashboardHeader } from "@/components/dashboard/dashboard-header"
import { DashboardMetrics } from "@/components/dashboard/dashboard-metrics"
import { ProductsOverview } from "@/components/dashboard/products-overview"
import { RecentActivity } from "@/components/dashboard/recent-activity"
import { WalletConnect } from "@/components/wallet-connect"

export default function DashboardPage() {
  return (
    <div className="flex flex-col gap-6 p-6 md:p-8">
      <DashboardHeader />
      <WalletConnect />
      <DashboardMetrics />
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <ProductsOverview />
        <RecentActivity />
      </div>
    </div>
  )
}

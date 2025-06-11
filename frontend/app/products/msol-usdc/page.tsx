import { ProductHeader } from "@/components/products/product-header"
import { RiskPoolSelector } from "@/components/products/risk-pool-selector"
import { DualStakingForm } from "@/components/products/dual-staking-form"
import { ProductInfo } from "@/components/products/product-info"
import { WalletConnect } from "@/components/wallet-connect"

export default function MsolUsdcProductPage() {
  return (
    <div className="flex flex-col gap-6 p-6 md:p-8">
      <ProductHeader
        title="mSOL-USDC Dual Staking"
        description="Stake both mSOL and USDC to earn enhanced yields with optional LP rewards"
        productType="msol-usdc"
      />
      <WalletConnect />
      <RiskPoolSelector productType="msol-usdc" />
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <DualStakingForm />
        <ProductInfo productType="msol-usdc" />
      </div>
    </div>
  )
}
